//! Original SolidCraft mark, shared by desktop, packaging and CLI.
/// This block is the icon image data
pub fn rgba(size: usize) -> Result<Vec<u8>, String> {
    if !(1..=2048).contains(&size) {
        return Err("icon size must be 1..=2048".into());
    }
    type P = (f32, f32);
    enum Paint {
        Solid([f32; 3]),
        /// Linear gradient in SVG user-space units (gradientUnits="userSpaceOnUse").
        Linear { from: P, to: P, c0: [f32; 3], c1: [f32; 3] },
    }
    fn hex(c: u32) -> [f32; 3] {
        [((c >> 16) & 255) as f32, ((c >> 8) & 255) as f32, (c & 255) as f32]
    }
    // Coverage buffer at 4x4 samples per pixel (2x2 for big sizes), averaged down at the end.
    let ss = if size > 512 { 2 } else { 4 };
    let n = size * ss;
    let mut buf = vec![[0u8; 4]; n * n];
    let s = n as f32 / 331.982; // SVG units -> samples: the logo fills the height
    let ox = (n as f32 - 295.0 * s) / 2.0; // and is centred sideways
    // Fill a simple (possibly concave) polygon given in SVG units, even-odd rule.
    let mut fill = |pts: &[P], paint: &Paint| {
        let v: Vec<P> = pts.iter().map(|p| (ox + p.0 * s, p.1 * s)).collect();
        let (top, bottom) = v.iter().fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.1), b.max(p.1)));
        let mut xs: Vec<f32> = Vec::new();
        for y in ((top - 0.5).ceil() as i32).max(0)..=((bottom - 0.5).floor() as i32).min(n as i32 - 1) {
            let cy = y as f32 + 0.5;
            xs.clear();
            for i in 0..v.len() {
                let (a, b) = (v[i], v[(i + 1) % v.len()]);
                if (a.1 <= cy) != (b.1 <= cy) {
                    xs.push(a.0 + (cy - a.1) / (b.1 - a.1) * (b.0 - a.0));
                }
            }
            xs.sort_by(|a, b| a.total_cmp(b));
            for pair in xs.chunks_exact(2) {
                for x in ((pair[0] - 0.5).ceil() as i32).max(0)..=((pair[1] - 0.5).floor() as i32).min(n as i32 - 1) {
                    let col = match paint {
                        Paint::Solid(c) => *c,
                        Paint::Linear { from, to, c0, c1 } => {
                            let (px, py) = ((x as f32 + 0.5 - ox) / s, cy / s);
                            let (dx, dy) = (to.0 - from.0, to.1 - from.1);
                            let t = (((px - from.0) * dx + (py - from.1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
                            [
                                c0[0] + (c1[0] - c0[0]) * t,
                                c0[1] + (c1[1] - c0[1]) * t,
                                c0[2] + (c1[2] - c0[2]) * t,
                            ]
                        }
                    };
                    buf[y as usize * n + x as usize] =
                        [col[0].round() as u8, col[1].round() as u8, col[2].round() as u8, 255];
                }
            }
        }
    };
    // Shapes, in the same paint order as the SVG.
    // Small dark-blue triangle (top right).
    fill(&[(160.0, 80.829), (220.0, 46.188), (220.0, 115.47)], &Paint::Solid(hex(0x1960c5)));
    // Small light-blue triangle (right).
    fill(&[(230.091, 202.073), (230.01, 132.796), (290.005, 167.434)], &Paint::Solid(hex(0x4aa6fd)));
    // Light blue face (top).
    fill(
        &[(9.995, 80.832), (150.0, 0.0), (215.0, 37.528), (140.0, 80.829), (215.0, 124.13), (149.901, 161.715)],
        &Paint::Linear { from: (112.497, 0.0), to: (112.497, 161.715), c0: hex(0x4aa6fd), c1: hex(0x3089f3) },
    );
    // Dark blue face (right).
    fill(
        &[
            (154.802, 170.433),
            (220.01, 132.785),
            (220.111, 219.393),
            (295.0, 176.098),
            (295.0, 251.147),
            (154.99, 331.982),
        ],
        &Paint::Linear { from: (140.052, 147.535), to: (309.75, 317.232), c0: hex(0x1960c5), c1: hex(0x15499c) },
    );
    // Medium blue face (left).
    fill(
        &[(5.0, 164.545), (5.0, 89.495), (144.802, 170.318), (144.99, 331.971), (80.0, 294.449), (80.0, 207.846)],
        &Paint::Linear { from: (74.995, 89.495), to: (74.995, 331.971), c0: hex(0x2380e1), c1: hex(0x1564dc) },
    );
    // Average each ss x ss block down to one pixel (straight alpha = covered share).
    let mut rgba = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let (mut sum, mut cnt) = ([0u32; 3], 0u32);
            for j in 0..ss {
                for i in 0..ss {
                    let p = buf[(y * ss + j) * n + x * ss + i];
                    if p[3] != 0 {
                        cnt += 1;
                        for k in 0..3 {
                            sum[k] += u32::from(p[k]);
                        }
                    }
                }
            }
            if cnt == 0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let a = 255 * cnt / (ss * ss) as u32;
                rgba.extend_from_slice(&[(sum[0] / cnt) as u8, (sum[1] / cnt) as u8, (sum[2] / cnt) as u8, a as u8]);
            }
        }
    }
    Ok(rgba)
}

#[cfg(test)]
mod tests {
    #[test]
    fn shared_mark_has_transparency_and_original_colors_at_window_and_package_sizes() {
        for size in [16, 24, 32, 48, 64, 128, 256] {
            let pixels = super::rgba(size).expect("valid size");
            assert_eq!(pixels.len(), size * size * 4);
            assert!(pixels.as_chunks::<4>().0.iter().any(|p| p[3] == 0));
            assert!(pixels.as_chunks::<4>().0.iter().any(|p| p[3] == 255 && p[2] > p[0]));
        }
        assert!(super::rgba(0).is_err());
        assert!(super::rgba(2049).is_err());
    }
}