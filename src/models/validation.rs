/// Validasi koordinat geografis berada dalam cakupan wilayah Kota Bandar Lampung & sekitarnya
pub fn validate_coordinates(lat: f64, lon: f64) -> Result<(), String> {
    if !(-6.0..=-5.0).contains(&lat) {
        return Err(format!(
            "Latitude {:.6} di luar cakupan Kota Bandar Lampung (rentang valid: -6.0 s/d -5.0)",
            lat
        ));
    }
    if !(104.5..=106.0).contains(&lon) {
        return Err(format!(
            "Longitude {:.6} di luar cakupan Kota Bandar Lampung (rentang valid: 104.5 s/d 106.0)",
            lon
        ));
    }
    Ok(())
}

/// Sanitasi string sederhana untuk mencegah XSS dasar
#[allow(dead_code)]
pub fn sanitize_text(input: &str) -> String {
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinates_bandar_lampung() {
        // Tugu Adipura Bandar Lampung
        assert!(validate_coordinates(-5.4215, 105.2605).is_ok());
        // Simpang MBK
        assert!(validate_coordinates(-5.3789, 105.2536).is_ok());
        // Di luar Lampung (misal Jakarta)
        assert!(validate_coordinates(-6.2088, 106.8456).is_err());
    }

    #[test]
    fn test_sanitize_text() {
        let dirty = "<script>alert('xss');</script>";
        let clean = sanitize_text(dirty);
        assert!(!clean.contains('<'));
        assert!(!clean.contains('>'));
    }
}
