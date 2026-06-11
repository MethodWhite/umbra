use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;


#[derive(Parser, Debug)]
#[command(name = "umbra", version, about = "UMBRA - AI Agent System")]
struct Cli {
    #[arg(long)]
    api_port: Option<u16>,

    #[arg(long)]
    api_host: Option<String>,

    #[arg(long)]
    frontend_port: Option<u16>,

    #[arg(long = "no-frontend")]
    no_frontend: bool,

    #[arg(long)]
    ssl: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    Start,
    Stop,
    Status,
}

fn load_tls_config(tls_dir: &PathBuf) -> Result<Arc<rustls::ServerConfig>> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(tls_dir).ok();

    let cert_path = tls_dir.join("cert.pem");
    let key_path = tls_dir.join("key.pem");

    if !cert_path.exists() || !key_path.exists() {
        tracing::warn!("Cert TLS no encontrado en {:?}. Usando cert auto-firmado.", tls_dir);
        let key_pair = rcgen::KeyPair::generate()?;
        let mut params = rcgen::CertificateParams::new(vec!["localhost".to_string()])?;
        params.subject_alt_names = vec![
            rcgen::SanType::DnsName("localhost".try_into().unwrap()),
            rcgen::SanType::IpAddress("127.0.0.1".parse().unwrap()),
        ];
        params.distinguished_name.push(rcgen::DnType::CommonName, "UMBRA");
        let cert = params.self_signed(&key_pair)?;
        std::fs::write(&cert_path, cert.pem())?;
        std::fs::write(&key_path, key_pair.serialize_pem())?;
        std::fs::set_permissions(&cert_path, std::fs::Permissions::from_mode(0o644))?;
        std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
    }

    let certs: Vec<rustls::pki_types::CertificateDer<'static>> = {
        let cert_pem = std::fs::read_to_string(&cert_path)?;
        let parsed = pem::parse_many(&cert_pem)?;
        parsed.into_iter()
            .map(|p| rustls::pki_types::CertificateDer::from(p.contents().to_vec()))
            .collect()
    };
    let key = {
        let key_pem = std::fs::read_to_string(&key_path)?;
        let parsed = pem::parse(&key_pem)?;
        let key_bytes = parsed.contents().to_vec();
        rustls::pki_types::PrivateKeyDer::try_from(key_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid private key DER: {}", e))?
    };
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| anyhow::anyhow!("TLS: {}", e))?;

    Ok(Arc::new(tls_config))
}

async fn serve_tls(
    router: axum::Router,
    listener: tokio::net::TcpListener,
    tls_config: Arc<rustls::ServerConfig>,
) -> Result<()> {
    use hyper_util::rt::TokioIo;
    use hyper_util::service::TowerToHyperService;
    use tokio_rustls::TlsAcceptor;
    use tower::Service;

    let tls_acceptor = TlsAcceptor::from(tls_config);
    let make_service = router.into_make_service();
    loop {
        let (stream, peer) = listener.accept().await?;
        let acceptor = tls_acceptor.clone();
        let mut make_service = make_service.clone();
        tokio::spawn(async move {
            let tls_stream = match acceptor.accept(stream).await {
                Ok(s) => TokioIo::new(s),
                Err(e) => { tracing::warn!("TLS: {}", e); return; }
            };
            let svc = match make_service.call(&peer).await {
                Ok(s) => s, Err(e) => { tracing::warn!("Svc: {}", e); return; }
            };
            let hyper_svc = TowerToHyperService::new(svc);
            if let Err(e) = hyper::server::conn::http1::Builder::new()
                .serve_connection(tls_stream, hyper_svc).await
            { tracing::warn!("Conn: {}", e); }
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let app = umbra::init().await?;
    let app = Arc::new(app);

    let backend_host = &app.config.api.backend_host;
    let backend_port = cli.api_port.unwrap_or(app.config.api.backend_port);
    let frontend_port = cli.frontend_port.unwrap_or(app.config.api.frontend_port);

    let host = cli.api_host.unwrap_or_else(|| backend_host.clone());

    // Build backend engine router
    let backend_router = umbra::api::server::build_router(
        Arc::new(app.agent.clone()),
        app.security.clone(),
        app.ironclaw.clone(),
        app.memory.clone(),
        app.audio.clone(),
        app.sub_agents.clone(),
        Arc::new(tokio::sync::Mutex::new(app.resource_manager.clone())),
        app.debugger.clone(),
        app.config.paths.models_dir.clone(),
    );

    // Build frontend router
    let frontend_state = umbra::frontend::FrontendState::new();
    let frontend_router = umbra::frontend::build_frontend_router(frontend_state);

    // Combined router: backend API + frontend API + static files
    let combined = backend_router.clone().merge(frontend_router);

    // Backend server (existing API)
    let backend_addr: SocketAddr = format!("{}:{}", host, backend_port).parse()?;
    let backend_listener = TcpListener::bind(&backend_addr).await?;
    tracing::info!("Backend API en http://{}", backend_addr);

    // Frontend server (combined API + UI)
    let frontend_addr: SocketAddr = format!("{}:{}", host, frontend_port).parse()?;
    let frontend_listener = TcpListener::bind(&frontend_addr).await?;
    tracing::info!("Frontend UI en http://{}", frontend_addr);

    if cli.ssl {
        let tls_dir = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
            .join(".umbra").join("tls");
        let tls_config = load_tls_config(&tls_dir)?;

        let backend = serve_tls(backend_router, backend_listener, tls_config.clone());
        let frontend = serve_tls(combined, frontend_listener, tls_config);
        tokio::select! {
            result = backend => result?,
            result = frontend => result?,
        }
    } else {
        let backend = axum::serve(backend_listener, backend_router);
        let frontend = axum::serve(frontend_listener, combined);
        tokio::select! {
            result = backend => result?,
            result = frontend => result?,
        }
    }

    Ok(())
}
