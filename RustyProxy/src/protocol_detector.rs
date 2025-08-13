pub enum Protocol {
    SSH,
    OpenVPN,
    V2Ray,
    Unknown,
}

impl Protocol {
    pub fn to_string(&self) -> &str {
        match self {
            Protocol::SSH => "SSH",
            Protocol::OpenVPN => "OpenVPN",
            Protocol::V2Ray => "V2Ray",
            Protocol::Unknown => "Unknown",
        }
    }
}

pub fn detect_protocol(data: &[u8]) -> Protocol {
    // SSH: O protocolo SSH começa com "SSH-"
    if data.starts_with(b"SSH-") {
        return Protocol::SSH;
    }

    // OpenVPN: Detecção simplificada
    // Pacotes OpenVPN UDP frequentemente começam com 0x20 ou 0x30 no primeiro byte.
    // Pacotes OpenVPN TCP podem ter 0x00 no primeiro byte e o segundo byte > 0x00.
    if data.len() >= 2 {
        let first_byte = data[0];
        let second_byte = data[1];

        // UDP patterns
        if (first_byte & 0xF0) == 0x20 || (first_byte & 0xF0) == 0x30 {
            return Protocol::OpenVPN;
        }
        
        // TCP patterns
        if first_byte == 0x00 && second_byte > 0x00 {
            return Protocol::OpenVPN;
        }
    }

    // V2Ray (VMess): Heurística simplificada
    // Baseado na análise anterior do MultiFlowPX, que verificava bytes altos e padrões específicos.
    if data.len() >= 16 {
        let mut high_byte_count = 0;
        for i in 0..16 {
            if data[i] > 0x7F {
                high_byte_count += 1;
            }
        }
        // Se houver *algum* byte alto nos primeiros 16 bytes, ou se o padrão 0x01 0x00 for encontrado.
        if high_byte_count > 0 || (data[0] == 0x01 && data[1] == 0x00) {
            return Protocol::V2Ray;
        }
    }

    Protocol::Unknown
}