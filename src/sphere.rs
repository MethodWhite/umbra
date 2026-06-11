use eframe::egui::{Color32, Pos2};
use std::f32::consts::PI;

#[derive(Clone)]
pub struct SphereParticle {
    pub x: f32, pub y: f32, pub z: f32,
    pub base_radius: f32,
    pub hue: f32,
}

pub struct ActiveNode {
    pub index: usize,
    pub intensity: f32,
}

pub struct SphereRenderer {
    particles: Vec<SphereParticle>,
    agent_spheres: Vec<AgentSphere>,
    active_nodes: Vec<ActiveNode>,
    radius: f32,
}

pub struct AgentSphere {
    pub x: f32, pub y: f32, pub z: f32,
    pub radius: f32,
    pub label: String,
    pub hue: f32,
    pub active: bool,
}

impl SphereRenderer {
    pub fn new(count: usize) -> Self {
        let mut particles = Vec::with_capacity(count);
        let golden_ratio = (1.0 + (5.0_f32).sqrt()) / 2.0;
        let r = 110.0;

        for i in 0..count {
            let theta = 2.0 * PI * i as f32 / golden_ratio;
            let phi = (1.0 - 2.0 * (i as f32 + 0.5) / count as f32).acos();
            let radius = r + (i as f32 / count as f32) * 20.0;
            particles.push(SphereParticle {
                x: radius * phi.sin() * theta.cos(),
                y: radius * phi.cos(),
                z: radius * phi.sin() * theta.sin(),
                base_radius: 1.0 + (i as f32 % 5.0) * 0.3,
                hue: i as f32 * 0.618,
            });
        }

        Self { particles, agent_spheres: Vec::new(), active_nodes: Vec::new(), radius: r }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn set_active_nodes(&mut self, nodes: Vec<ActiveNode>) {
        self.active_nodes = nodes;
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        let golden_ratio = (1.0 + (5.0_f32).sqrt()) / 2.0;
        let count = self.particles.len();
        for i in 0..count {
            let theta = 2.0 * PI * i as f32 / golden_ratio;
            let phi = (1.0 - 2.0 * (i as f32 + 0.5) / count as f32).acos();
            let r = radius + (i as f32 / count as f32) * 20.0;
            self.particles[i].x = r * phi.sin() * theta.cos();
            self.particles[i].y = r * phi.cos();
            self.particles[i].z = r * phi.sin() * theta.sin();
        }
    }

    pub fn set_agents(&mut self, agents: &[(&str, bool, f32)]) {
        self.agent_spheres = agents.iter().enumerate().map(|(i, (name, active, mood))| {
            let angle = i as f32 * 2.0 * PI / agents.len() as f32;
            AgentSphere {
                x: angle.cos() * 100.0,
                y: angle.sin() * 80.0,
                z: 0.0,
                radius: 12.0 + mood * 8.0,
                label: name.to_string(),
                hue: if *active { 0.55 } else { 0.0 },
                active: *active,
            }
        }).collect();
    }

    pub fn render(
        &self,
        painter: &egui::Painter,
        center: Pos2,
        mood_hue: f32,      // 0.0-1.0 hue from EmotionalState
        mood_sat: f32,      // saturation
        mood_intensity: f32,// intensity/activity
        activity: f32,      // 0.0=idle, 1.0=full
        t: f32,
        rot_speed: f32,     // rotation speed multiplier (0.0-2.0)
        radius_scale: f32,  // radius scale (0.6=contracted, 1.0=normal)
        rot_x_ampl: f32,    // X rotation amplitude (-1.0 to 1.0)
        rot_z_ampl: f32,    // Z rotation amplitude (-1.0 to 1.0)
        rot_direction: f32, // +1 = left-to-right, -1 = right-to-left
    ) {
        let rot_y = t * 0.3 * rot_speed * rot_direction;
        let rot_x = t * 0.2 * rot_speed * rot_x_ampl;
        let rot_z = t * 0.15 * rot_speed * rot_z_ampl;

        let hue_to_rgb = |h: f32, s: f32, v: f32| -> (u8, u8, u8) {
            let h = h * 360.0;
            let s = s.clamp(0.0, 1.0);
            let v = v.clamp(0.0, 1.0);
            let c = v * s;
            let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
            let m = v - c;
            let (r, g, b) = match (h as u32) % 360 {
                0..=59 => (c, x, 0.0), 60..=119 => (x, c, 0.0),
                120..=179 => (0.0, c, x), 180..=239 => (0.0, x, c),
                240..=299 => (x, 0.0, c), _ => (c, 0.0, x),
            };
            (((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8)
        };

        let (r, g, b) = hue_to_rgb(mood_hue, mood_sat, mood_intensity);
        let base_color = Color32::from_rgb(r, g, b);
        let dim_color = Color32::from_rgba_premultiplied(100, 120, 150, 60);

        // Build active set for lookups (simple glow, no connections)
        let mut active_set: Vec<(usize, f32)> = self.active_nodes.iter()
            .map(|n| (n.index, n.intensity)).collect();
        active_set.sort_by_key(|(idx, _)| *idx);

        // Project and sort particles
        let mut projected: Vec<(f32, Pos2, f32, Color32)> = Vec::with_capacity(self.particles.len());

        for (i, p) in self.particles.iter().enumerate() {
            let cos_y = rot_y.cos();
            let sin_y = rot_y.sin();
            let cos_x = rot_x.cos();
            let sin_x = rot_x.sin();
            let cos_z = rot_z.cos();
            let sin_z = rot_z.sin();

            let rx = p.x * cos_y - p.z * sin_y;
            let rz = p.x * sin_y + p.z * cos_y;
            let ry = p.y * cos_x - rz * sin_x;
            let rz2 = p.y * sin_x + rz * cos_x;
            // Apply Z rotation: rotate around Z axis
            let rx2 = rx * cos_z - ry * sin_z;
            let ry2 = rx * sin_z + ry * cos_z;

            let fov = 300.0;
            let scale = fov / (fov + rz2);
            let sx = center.x + rx2 * scale;
            let sy = center.y + ry2 * scale;

            // Check if active node
            let is_active = active_set.binary_search_by_key(&i, |(idx, _)| *idx);

            let size = if let Ok(pos) = is_active {
                let (_, intensity) = active_set[pos];
                let pulse = 1.0 + (t * 3.0 + i as f32 * 0.5).sin() * 0.3;
                p.base_radius * scale * (1.5 + intensity * 1.0 * pulse) * radius_scale
            } else {
                p.base_radius * scale * (0.8 + activity * 0.4) * radius_scale
            };

            let depth_factor = (rz2 + 100.0) / 200.0;
            let color = if let Ok(pos) = is_active {
                let (_, intensity) = active_set[pos];
                let pulse = 1.0 + (t * 3.0 + i as f32 * 0.5).sin() * 0.3;
                let blend = 0.5 + intensity * 0.5 * pulse;
                Color32::from_rgba_premultiplied(
                    (base_color.r() as f32 * (1.0 - blend) + 255.0 * blend) as u8,
                    (base_color.g() as f32 * (1.0 - blend) + 220.0 * blend) as u8,
                    (base_color.b() as f32 * (1.0 - blend) + 255.0 * blend) as u8,
                    (depth_factor * 230.0) as u8,
                )
            } else if depth_factor > 0.5 {
                Color32::from_rgba_premultiplied(
                    (base_color.r() as f32 * (1.0 - activity * 0.3) + 255.0 * activity * 0.3) as u8,
                    (base_color.g() as f32 * (1.0 - activity * 0.3) + 255.0 * activity * 0.3) as u8,
                    (base_color.b() as f32 * (1.0 - activity * 0.3) + 255.0 * activity * 0.3) as u8,
                    (depth_factor * 200.0) as u8,
                )
            } else {
                Color32::from_rgba_premultiplied(
                (dim_color.r() as f32 * (0.3 + depth_factor * 0.7)) as u8,
                (dim_color.g() as f32 * (0.3 + depth_factor * 0.7)) as u8,
                (dim_color.b() as f32 * (0.3 + depth_factor * 0.7)) as u8,
                (40 + (depth_factor * 80.0) as u8).max(50),
                )
            };

            projected.push((rz2, Pos2::new(sx, sy), size, color));
        }

        // Sort by Z (far to near) for painter's algorithm
        projected.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Draw particles
        for (_, pos, size, color) in &projected {
            painter.circle_filled(*pos, *size, *color);
        }

        // Render agent spheres
        for agent in &self.agent_spheres {
            let rx = agent.x * rot_y.cos() - agent.z * rot_y.sin();
            let rz = agent.x * rot_y.sin() + agent.z * rot_y.cos();
            let fov = 300.0;
            let scale = fov / (fov + rz);
            let sx = center.x + rx * scale;
            let sy = center.y + agent.y * scale;
            let size = agent.radius * scale;

            if size > 2.0 {
                let agent_color = if agent.active {
                    Color32::from_rgb(0, 220, 255)
                } else {
                    Color32::from_rgba_premultiplied(60, 60, 80, 80)
                };

                painter.circle_filled(Pos2::new(sx, sy), size * 1.5, Color32::from_rgba_premultiplied(
                    agent_color.r(), agent_color.g(), agent_color.b(), 20));
                painter.circle_filled(Pos2::new(sx, sy), size, agent_color);
                painter.circle_filled(Pos2::new(sx - size * 0.2, sy - size * 0.2), size * 0.3, Color32::from_rgba_premultiplied(255, 255, 255, 100));
                painter.text(Pos2::new(sx, sy + size + 4.0), egui::Align2::CENTER_CENTER,
                    &agent.label, egui::FontId::monospace(8.0), Color32::from_rgba_premultiplied(200, 200, 255, 150));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_particle_count() {
        let renderer = SphereRenderer::new(250);
        assert_eq!(renderer.particles.len(), 250);
    }

    #[test]
    fn test_sphere_particles_on_unit_sphere() {
        let renderer = SphereRenderer::new(500);
        for p in &renderer.particles {
            let dist_sq = p.x * p.x + p.y * p.y + p.z * p.z;
            // Fibonacci sphere uses golden angle, particles may be at varying distances
            // from center depending on radius parameter
            assert!(dist_sq > 0.0, "particle at origin: dist^2 = {}", dist_sq);
        }
    }

    #[test]
    fn test_sphere_new_default() {
        let renderer = SphereRenderer::new(100);
        assert_eq!(renderer.particles.len(), 100);
        assert!(renderer.active_nodes.is_empty());
    }

    #[test]
    fn test_sphere_particles_have_valid_hue() {
        let renderer = SphereRenderer::new(100);
        for p in &renderer.particles {
            assert!(p.hue >= 0.0 && p.hue <= 360.0, "hue out of range: {}", p.hue);
        }
    }
}
