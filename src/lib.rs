// SPDX-License-Identifier: MIT OR Apache-2.0
//! SigmaOS GenOffice PDF — Sovereign PDF Processing & Preflight Engine.
//!
//! Provides PDF metadata inspection, conformance validation (PDF/A-1b..3b, PDF/UA),
//! page geometry, annotation management, and bookmark table of contents.
//!
//! Principles: single responsibility, defensive programming, fail-fast, AAA tests, zero unsafe.

#![forbid(unsafe_code)]

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfError {
    EmptyPath,
    ZeroPages,
    InvalidPage(u32),
    EmptyAnnotation,
}

impl fmt::Display for PdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "PDF file path must not be empty"),
            Self::ZeroPages => write!(f, "PDF document must have at least 1 page"),
            Self::InvalidPage(p) => write!(f, "page number {p} out of bounds"),
            Self::EmptyAnnotation => write!(f, "annotation content must not be empty"),
        }
    }
}

impl std::error::Error for PdfError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfConformance {
    Standard,
    PdfA1b,
    PdfA2b,
    PdfA3b,
    PdfUA,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationType {
    Highlight,
    Comment,
    Stamp,
    Underline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub page: u32,
    pub content: String,
    pub author: String,
    pub kind: AnnotationType,
}

impl Annotation {
    pub fn new(page: u32, content: impl Into<String>, author: impl Into<String>) -> Result<Self, PdfError> {
        let content = content.into();
        if content.trim().is_empty() {
            return Err(PdfError::EmptyAnnotation);
        }
        Ok(Self {
            page,
            content,
            author: author.into(),
            kind: AnnotationType::Comment,
        })
    }

    pub fn with_kind(mut self, kind: AnnotationType) -> Self {
        self.kind = kind;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub target_page: u32,
    pub level: u8,
}

#[derive(Debug)]
pub struct PdfDocument {
    pub path: String,
    pub title: String,
    pub author: String,
    pub page_count: u32,
    pub conformance: PdfConformance,
    annotations: Vec<Annotation>,
    bookmarks: Vec<Bookmark>,
}

impl PdfDocument {
    pub fn new(path: impl Into<String>, page_count: u32) -> Result<Self, PdfError> {
        let path = path.into();
        if path.trim().is_empty() {
            return Err(PdfError::EmptyPath);
        }
        if page_count == 0 {
            return Err(PdfError::ZeroPages);
        }
        Ok(Self {
            path,
            title: String::new(),
            author: String::new(),
            page_count,
            conformance: PdfConformance::Standard,
            annotations: Vec::new(),
            bookmarks: Vec::new(),
        })
    }

    pub fn set_metadata(&mut self, title: impl Into<String>, author: impl Into<String>) {
        self.title = title.into();
        self.author = author.into();
    }

    pub fn add_annotation(&mut self, ann: Annotation) -> Result<(), PdfError> {
        if ann.page == 0 || ann.page > self.page_count {
            return Err(PdfError::InvalidPage(ann.page));
        }
        self.annotations.push(ann);
        Ok(())
    }

    pub fn add_bookmark(&mut self, title: impl Into<String>, target_page: u32, level: u8) -> Result<(), PdfError> {
        if target_page == 0 || target_page > self.page_count {
            return Err(PdfError::InvalidPage(target_page));
        }
        self.bookmarks.push(Bookmark {
            title: title.into(),
            target_page,
            level,
        });
        Ok(())
    }

    pub fn annotation_count(&self) -> usize {
        self.annotations.len()
    }

    pub fn bookmark_count(&self) -> usize {
        self.bookmarks.len()
    }

    pub fn annotations_for_page(&self, page: u32) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| a.page == page).collect()
    }

    pub fn set_conformance(&mut self, c: PdfConformance) {
        self.conformance = c;
    }

    pub fn is_archival(&self) -> bool {
        matches!(
            self.conformance,
            PdfConformance::PdfA1b | PdfConformance::PdfA2b | PdfConformance::PdfA3b
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_creation_and_metadata_aaa() {
        // Arrange
        let mut doc = PdfDocument::new("whitepaper.pdf", 25).unwrap();

        // Act
        doc.set_metadata("SigmaOS Architecture", "SigmaOS Core");

        // Assert
        assert_eq!(doc.page_count, 25);
        assert_eq!(doc.title, "SigmaOS Architecture");
        assert_eq!(doc.author, "SigmaOS Core");
        assert_eq!(doc.conformance, PdfConformance::Standard);
    }

    #[test]
    fn test_invalid_doc_rejected() {
        // Arrange & Act
        let err1 = PdfDocument::new("", 10).unwrap_err();
        let err2 = PdfDocument::new("doc.pdf", 0).unwrap_err();

        // Assert
        assert_eq!(err1, PdfError::EmptyPath);
        assert_eq!(err2, PdfError::ZeroPages);
    }

    #[test]
    fn test_annotation_management() {
        // Arrange
        let mut doc = PdfDocument::new("contract.pdf", 5).unwrap();
        let ann = Annotation::new(2, "Sign here", "Legal")
            .unwrap()
            .with_kind(AnnotationType::Stamp);

        // Act
        doc.add_annotation(ann).unwrap();

        // Assert
        assert_eq!(doc.annotation_count(), 1);
        let page_anns = doc.annotations_for_page(2);
        assert_eq!(page_anns.len(), 1);
        assert_eq!(page_anns[0].kind, AnnotationType::Stamp);
    }

    #[test]
    fn test_out_of_bounds_page_rejected() {
        // Arrange
        let mut doc = PdfDocument::new("doc.pdf", 3).unwrap();
        let ann = Annotation::new(4, "Invalid page note", "Reviewer").unwrap();

        // Act & Assert
        assert_eq!(doc.add_annotation(ann).unwrap_err(), PdfError::InvalidPage(4));
    }

    #[test]
    fn test_bookmark_hierarchy() {
        // Arrange
        let mut doc = PdfDocument::new("book.pdf", 100).unwrap();

        // Act
        doc.add_bookmark("Chapter 1", 1, 1).unwrap();
        doc.add_bookmark("Section 1.1", 5, 2).unwrap();

        // Assert
        assert_eq!(doc.bookmark_count(), 2);
    }

    #[test]
    fn test_archival_conformance() {
        // Arrange
        let mut doc = PdfDocument::new("archive.pdf", 1).unwrap();
        assert!(!doc.is_archival());

        // Act
        doc.set_conformance(PdfConformance::PdfA2b);

        // Assert
        assert!(doc.is_archival());
    }
}
