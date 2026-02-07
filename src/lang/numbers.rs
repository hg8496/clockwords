/// Parse a number word or digit string into a u32.
/// Tries the digit parse first, then each language table.
pub fn parse_number(s: &str) -> Option<u32> {
    // Try digit parse first
    if let Ok(n) = s.parse::<u32>() {
        return Some(n);
    }

    let lower = s.to_lowercase();
    parse_number_en(&lower)
        .or_else(|| parse_number_de(&lower))
        .or_else(|| parse_number_fr(&lower))
        .or_else(|| parse_number_es(&lower))
}

pub fn parse_number_en(s: &str) -> Option<u32> {
    match s {
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "thirteen" => Some(13),
        "fourteen" => Some(14),
        "fifteen" => Some(15),
        "sixteen" => Some(16),
        "seventeen" => Some(17),
        "eighteen" => Some(18),
        "nineteen" => Some(19),
        "twenty" => Some(20),
        "thirty" => Some(30),
        _ => None,
    }
}

pub fn parse_number_de(s: &str) -> Option<u32> {
    match s {
        "ein" | "eins" | "eine" | "einem" | "einen" => Some(1),
        "zwei" => Some(2),
        "drei" => Some(3),
        "vier" => Some(4),
        "fünf" | "fuenf" | "funf" => Some(5),
        "sechs" => Some(6),
        "sieben" => Some(7),
        "acht" => Some(8),
        "neun" => Some(9),
        "zehn" => Some(10),
        "elf" => Some(11),
        "zwölf" | "zwoelf" => Some(12),
        "dreizehn" => Some(13),
        "vierzehn" => Some(14),
        "fünfzehn" | "fuenfzehn" => Some(15),
        "sechzehn" => Some(16),
        "siebzehn" => Some(17),
        "achtzehn" => Some(18),
        "neunzehn" => Some(19),
        "zwanzig" => Some(20),
        "dreißig" | "dreissig" => Some(30),
        _ => None,
    }
}

pub fn parse_number_fr(s: &str) -> Option<u32> {
    match s {
        "un" | "une" => Some(1),
        "deux" => Some(2),
        "trois" => Some(3),
        "quatre" => Some(4),
        "cinq" => Some(5),
        "six" => Some(6),
        "sept" => Some(7),
        "huit" => Some(8),
        "neuf" => Some(9),
        "dix" => Some(10),
        "onze" => Some(11),
        "douze" => Some(12),
        "treize" => Some(13),
        "quatorze" => Some(14),
        "quinze" => Some(15),
        "seize" => Some(16),
        "vingt" => Some(20),
        "trente" => Some(30),
        _ => None,
    }
}

pub fn parse_number_es(s: &str) -> Option<u32> {
    match s {
        "un" | "uno" | "una" => Some(1),
        "dos" => Some(2),
        "tres" => Some(3),
        "cuatro" => Some(4),
        "cinco" => Some(5),
        "seis" => Some(6),
        "siete" => Some(7),
        "ocho" => Some(8),
        "nueve" => Some(9),
        "diez" => Some(10),
        "once" => Some(11),
        "doce" => Some(12),
        "trece" => Some(13),
        "catorce" => Some(14),
        "quince" => Some(15),
        "veinte" => Some(20),
        "treinta" => Some(30),
        _ => None,
    }
}
