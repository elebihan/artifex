//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use chrono::prelude::*;
use std::io::Write;

use crate::command::{Command, CommandOutput, CommandStatus};
use crate::error::Error;

/// Hold information about the execution of a command.
#[derive(Debug)]
pub struct ReportEntry {
    pub command: Command,
    pub status: CommandStatus,
}

#[derive(Debug)]
pub struct BatchReport {
    date: DateTime<Utc>,
    title: String,
    entries: Vec<ReportEntry>,
}

impl BatchReport {
    /// Create a new report.
    pub(crate) fn new(title: &str) -> Self {
        Self {
            date: Utc::now(),
            title: title.to_string(),
            entries: vec![],
        }
    }

    /// Append a new report entry with information about the execution of a command.
    pub(crate) fn push(&mut self, entry: ReportEntry) {
        self.entries.push(entry);
    }

    /// Return the creation date of a `Report`.
    pub fn date(&self) -> &DateTime<Utc> {
        &self.date
    }

    /// Return the entries in a `Report`.
    pub fn entries(&self) -> &[ReportEntry] {
        &self.entries
    }

    /// Return the title of a `Report`.
    pub fn title(&self) -> &str {
        &self.title
    }

    #[cfg(test)]
    pub(crate) fn spoof_date(&mut self, date: &str) -> Result<(), chrono::format::ParseError> {
        let date = DateTime::parse_from_rfc3339(date)?;
        self.date = date.into();
        Ok(())
    }
}

/// Convert a `Report` to a text representation using a markup format.
#[derive(Debug)]
pub struct MarkupReportRenderer {
    markup_kind: MarkupKind,
}

/// Kind of supported markup formats.
#[derive(Debug)]
pub enum MarkupKind {
    Xml,
    Yaml,
}

impl MarkupReportRenderer {
    /// Create a new report renderer.
    pub fn new(markup_kind: MarkupKind) -> Self {
        Self { markup_kind }
    }

    /// Render a report.
    pub fn render<W: Write>(&self, writer: &mut W, report: &BatchReport) -> Result<(), Error> {
        match self.markup_kind {
            MarkupKind::Xml => Self::render_as_xml(writer, report),
            MarkupKind::Yaml => Self::render_as_yaml(writer, report),
        }
    }

    fn render_as_yaml<W: Write>(writer: &mut W, report: &BatchReport) -> Result<(), Error> {
        writeln!(writer, "# Artifex batch report")?;
        write!(
            writer,
            "title   : {}\ndate    : {}\ncommands:\n",
            report.title(),
            report.date().to_rfc3339()
        )?;
        for entry in report.entries() {
            writeln!(writer, "- command: '{}'", entry.command)?;
            let (status, output) = match &entry.status {
                CommandStatus::Failure => ("failure", None),
                CommandStatus::Success(output) => ("success", output.as_ref()),
            };
            writeln!(writer, "  status : {}", status)?;
            if let Some(output) = output {
                writeln!(writer, "  output : |")?;
                match output {
                    CommandOutput::String(text) => {
                        for line in text.lines() {
                            writeln!(writer, "    {}", line)?;
                        }
                    }
                    CommandOutput::Uint32(number) => {
                        writeln!(writer, "    {}", number)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn render_as_xml<W: Write>(writer: &mut W, report: &BatchReport) -> Result<(), Error> {
        writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(writer, "<report>")?;
        writeln!(
            writer,
            "  <title>{}</title>\n  <date>{}</date>\n  <commands>",
            report.title(),
            report.date().to_rfc3339()
        )?;
        for entry in &report.entries {
            writeln!(
                writer,
                "    <command>\n      <input><![CDATA[{}]]></input>",
                entry.command
            )?;
            let (status, output) = match &entry.status {
                CommandStatus::Failure => ("failure", None),
                CommandStatus::Success(output) => ("success", output.as_ref()),
            };
            writeln!(writer, "      <status>{}</status>", status)?;
            if let Some(output) = output {
                match output {
                    CommandOutput::String(text) => {
                        writeln!(writer, "      <output><![CDATA[{}]]></output>", text)?;
                    }
                    CommandOutput::Uint32(number) => {
                        writeln!(writer, "      <output>{}</output>", number)?;
                    }
                };
            }
            writeln!(writer, "    </command>")?;
        }
        writeln!(writer, "  </commands>")?;
        writeln!(writer, "</report>")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandOutput;

    fn setup_report() -> BatchReport {
        let mut report = BatchReport::new("Dummy Report");
        report
            .spoof_date("2023-05-07T09:17:58.133639582+00:00")
            .unwrap();
        report.push(ReportEntry {
            command: Command::Execute("date -u".to_string()),
            status: CommandStatus::Success(Some(CommandOutput::String(
                "Sun May  7 09:17:58 UTC 2023".to_string(),
            ))),
        });
        report.push(ReportEntry {
            command: Command::Upgrade,
            status: CommandStatus::Failure,
        });
        report
    }

    fn render_to_markup(markup_kind: MarkupKind, reference: &str) {
        let report = setup_report();
        let renderer = MarkupReportRenderer::new(markup_kind);
        let mut buffer: Vec<u8> = vec![];
        let res = renderer.render(&mut buffer, &report);
        assert!(res.is_ok());
        let text = String::from_utf8(buffer).unwrap();
        assert_eq!(reference, text);
    }

    const REPORT_YAML: &str = r#"# Artifex batch report
title   : Dummy Report
date    : 2023-05-07T09:17:58.133639582+00:00
commands:
- command: 'EXECUTE: date -u'
  status : success
  output : |
    Sun May  7 09:17:58 UTC 2023
- command: 'UPGRADE'
  status : failure
"#;
    #[test]
    fn render_to_yaml() {
        render_to_markup(MarkupKind::Yaml, REPORT_YAML);
    }

    const REPORT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<report>
  <title>Dummy Report</title>
  <date>2023-05-07T09:17:58.133639582+00:00</date>
  <commands>
    <command>
      <input><![CDATA[EXECUTE: date -u]]></input>
      <status>success</status>
      <output><![CDATA[Sun May  7 09:17:58 UTC 2023]]></output>
    </command>
    <command>
      <input><![CDATA[UPGRADE]]></input>
      <status>failure</status>
    </command>
  </commands>
</report>
"#;

    #[test]
    fn render_to_xml() {
        render_to_markup(MarkupKind::Xml, REPORT_XML);
    }
}
