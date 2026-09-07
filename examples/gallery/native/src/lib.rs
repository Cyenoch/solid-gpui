use solid_gpui::{native_module, native_type};

#[native_type]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceStatus {
    Blocked,
    Ready,
    Established,
}

#[native_type]
#[serde(rename_all = "camelCase")]
pub struct BuildProgress {
    pub completed: u32,
    pub target: u32,
    pub percent: u32,
}

#[native_type]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceReport {
    pub normalized_name: String,
    pub slug: String,
    pub status: WorkspaceStatus,
    pub progress: BuildProgress,
    pub recommendations: Vec<String>,
    pub next_milestone: Option<u32>,
}

#[native_module(name = "gallery")]
mod app {
    use super::*;
    #[component(children = false)]
    pub fn build_badge(
        builds: u32,
        cx: &mut solid_gpui::native::ElementContext,
    ) -> impl solid_gpui::gpui::IntoElement {
        use solid_gpui::gpui::{InteractiveElement, ParentElement, Styled, div, px, rgb};
        let unit = if builds == 1 { "build" } else { "builds" };
        div()
            .id(cx.id())
            .px(px(10.))
            .py(px(6.))
            .rounded_md()
            .bg(rgb(0x1e3a5f))
            .text_color(rgb(0xdbeafe))
            .child(format!("Rust component · {builds} {unit}"))
    }
    #[command]
    pub fn analyze_workspace(
        name: String,
        readiness: bool,
        builds: u32,
    ) -> Result<WorkspaceReport, String> {
        let normalized_name = name.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized_name.is_empty() {
            return Err("Workspace name must not be blank".into());
        }
        if normalized_name.chars().count() > 80 {
            return Err("Workspace name must contain at most 80 characters".into());
        }
        if normalized_name.chars().any(char::is_control) {
            return Err("Workspace name must not contain control characters".into());
        }
        let mut slug = String::with_capacity(normalized_name.len());
        for character in normalized_name.chars().flat_map(char::to_lowercase) {
            if character.is_alphanumeric() {
                slug.push(character);
            } else if !slug.is_empty() && !slug.ends_with('-') {
                slug.push('-');
            }
        }
        if slug.ends_with('-') {
            slug.pop();
        }
        if slug.is_empty() {
            return Err("Workspace name must contain a letter or number".into());
        }
        let completed = builds.min(5);
        let status = if !readiness {
            WorkspaceStatus::Blocked
        } else if completed == 5 {
            WorkspaceStatus::Established
        } else {
            WorkspaceStatus::Ready
        };
        let recommendations = if !readiness {
            vec!["Complete the readiness checklist before building.".into()]
        } else if builds == 0 {
            vec![format!("Run the first build for {normalized_name}.")]
        } else if completed < 5 {
            let remaining = 5 - completed;
            let unit = if remaining == 1 { "build" } else { "builds" };
            vec![format!(
                "Complete {remaining} more {unit} to establish this workspace."
            )]
        } else {
            vec!["All build milestones are complete; review the release checklist.".into()]
        };
        Ok(WorkspaceReport {
            normalized_name,
            slug,
            status,
            progress: BuildProgress {
                completed,
                target: 5,
                percent: completed * 20,
            },
            recommendations,
            next_milestone: (completed < 5).then_some(completed + 1),
        })
    }
}

pub fn native_module() -> solid_gpui::native::ModuleDefinition {
    app::native_module()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_workspace_and_tracks_milestones() {
        let report = app::analyze_workspace("  Rust   Studio  ".into(), true, 3).unwrap();
        assert_eq!(report.normalized_name, "Rust Studio");
        assert_eq!(report.slug, "rust-studio");
        assert!(matches!(report.status, WorkspaceStatus::Ready));
        assert_eq!(report.progress.percent, 60);
        assert_eq!(report.next_milestone, Some(4));
        let blocked = app::analyze_workspace("demo".into(), false, u32::MAX).unwrap();
        assert!(matches!(blocked.status, WorkspaceStatus::Blocked));
        assert_eq!(blocked.progress.percent, 100);
        assert_eq!(blocked.next_milestone, None);
        assert!(app::analyze_workspace("  ".into(), true, 0).is_err());
    }
}
