use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhiType {
    Name,
    Email,
    Phone,
    Url,
    Mrn,
    Insurance,
    Date,
    Address,
    PostalCode,
    Id,
    Other,
}

#[derive(Debug, Clone)]
pub struct PhiRule {
    pub phi_type: PhiType,
    pub regex: Regex,
    pub label: &'static str,
}

pub fn default_rules() -> Vec<PhiRule> {
    vec![
        PhiRule {
            phi_type: PhiType::Email,
            regex: Regex::new(r"(?i)[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}").unwrap(),
            label: "email",
        },
        PhiRule {
            phi_type: PhiType::Phone,
            regex: Regex::new(r"(?:\+?\d{1,3}\s*)?(?:\(?\d{2,4}\)?[\s.-]?)\d{3,4}[\s.-]?\d{3,4}")
                .unwrap(),
            label: "phone",
        },
        PhiRule {
            phi_type: PhiType::Url,
            regex: Regex::new(r"https?://[^\s]+|www\.[^\s]+|\b\w+\.\w{2,}\b").unwrap(),
            label: "url",
        },
        PhiRule {
            phi_type: PhiType::Mrn,
            regex: Regex::new(r"(?i)\b(MRN|Record|Account)\s*[:#]?\s*\d{5,}\b").unwrap(),
            label: "mrn",
        },
        PhiRule {
            phi_type: PhiType::Insurance,
            regex: Regex::new(r"(?i)\b(Insurance|Policy)\s*[:#]?\s*[A-Z0-9-]{6,}\b").unwrap(),
            label: "insurance",
        },
        PhiRule {
            phi_type: PhiType::Date,
            regex: Regex::new(r"\b\d{1,2}[\/\-]\d{1,2}[\/\-]\d{2,4}\b|\b\d{4}[\/\-]\d{1,2}[\/\-]\d{1,2}\b")
                .unwrap(),
            label: "date",
        },
        PhiRule {
            phi_type: PhiType::Address,
            regex: Regex::new(r"\b\d{1,5}\s+[A-Za-z0-9.'-]+(?:\s+[A-Za-z0-9.'-]+){0,4}\s+(Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Lane|Ln|Drive|Dr|Court|Ct|Way|Place|Pl)\b")
                .unwrap(),
            label: "address",
        },
        PhiRule {
            phi_type: PhiType::PostalCode,
            regex: Regex::new(r"\b\d{5}(?:-\d{4})?\b|\b\d{4}\s?[A-Z]{2}\b").unwrap(),
            label: "postal_code",
        },
        PhiRule {
            phi_type: PhiType::Id,
            regex: Regex::new(r"\b\d{6,}\b").unwrap(),
            label: "id",
        },
    ]
}

pub fn phi_label(phi: PhiType) -> &'static str {
    match phi {
        PhiType::Name => "name",
        PhiType::Email => "email",
        PhiType::Phone => "phone",
        PhiType::Url => "url",
        PhiType::Mrn => "mrn",
        PhiType::Insurance => "insurance",
        PhiType::Date => "date",
        PhiType::Address => "address",
        PhiType::PostalCode => "postal_code",
        PhiType::Id => "id",
        PhiType::Other => "other",
    }
}
