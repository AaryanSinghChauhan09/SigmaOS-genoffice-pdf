#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfConformance { None, PdfA1b, PdfA2b, PdfA3b }

#[derive(Debug, Clone)]
pub struct Annotation {
    pub page: u32,
    pub content: String,
    pub author: String,
}

impl Annotation {
    pub fn new(page: u32, content: impl Into<String>, author: impl Into<String>) -> Self {
        Self { page, content: content.into(), author: author.into() }
    }
}

#[derive(Debug)]
pub struct PdfDocument {
    pub path: String,
    pub page_count: u32,
    pub conformance: PdfConformance,
    annotations: Vec<Annotation>,
}

impl PdfDocument {
    pub fn new(path: impl Into<String>, page_count: u32) -> Self {
        Self { path: path.into(), page_count, conformance: PdfConformance::None, annotations: Vec::new() }
    }
    pub fn add_annotation(&mut self, ann: Annotation) { self.annotations.push(ann); }
    pub fn annotation_count(&self) -> usize { self.annotations.len() }
    pub fn set_conformance(&mut self, c: PdfConformance) { self.conformance = c; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pdf_annotation_tracking() {
        let mut doc = PdfDocument::new("report.pdf", 10);
        doc.add_annotation(Annotation::new(1, "Review this", "alice"));
        assert_eq!(doc.annotation_count(), 1);
    }
}
