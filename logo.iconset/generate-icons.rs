// Placeholder: generates PNG icons from logo.svg at various sizes.
// Requires: `cargo install resvg` or `npm install -g sharp-cli`
//
// Usage with resvg/inkscape:
//   for size in 16 32 64 128 256 512; do
//     inkscape -w $size -h $size ../logo.svg -o icon_${size}x${size}.png
//   done
//
// On macOS, use iconutil:
//   mkdir -p Umbra.iconset
//   for size in 16 32 64 128 256 512; do
//     sips -z $size $size ../logo.svg --out Umbra.iconset/icon_${size}x${size}.png
//   done
//   iconutil -c icns Umbra.iconset
//
// On Linux with sharp:
//   npm install -g sharp-cli
//   for size in 16 32 64 128 256 512; do
//     sharp -i ../logo.svg -o icon_${size}x${size}.png resize $size $size
//   done
fn main() {
    println!("UMBRA Icon Generator");
    println!("Run: inkscape -w <size> -h <size> ../logo.svg -o icon_<size>x<size>.png");
}
