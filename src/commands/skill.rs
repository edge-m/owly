use std::fs;
use std::path::PathBuf;

use clap::Args as ClapArgs;

const SKILL_NAME: &str = "owly";
const SKILLS: &[(&str, &str)] = &[
    ("using-wiki/SKILL.md", include_str!("../../docs/i18n/en/skills/owly/using-wiki/SKILL.md")),
    ("wiki-config/SKILL.md", include_str!("../../docs/i18n/en/skills/owly/wiki-config/SKILL.md")),
    ("wiki-overview/SKILL.md", include_str!("../../docs/i18n/en/skills/owly/wiki-overview/SKILL.md")),
    ("wiki-ingest/SKILL.md", include_str!("../../docs/i18n/en/skills/owly/wiki-ingest/SKILL.md")),
    ("wiki-maintenance/SKILL.md", include_str!("../../docs/i18n/en/skills/owly/wiki-maintenance/SKILL.md")),
    ("wiki-search/SKILL.md", include_str!("../../docs/i18n/en/skills/owly/wiki-search/SKILL.md")),
];

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(help = "Directory where the Skill should be written")]
    pub output_directory: PathBuf,
}

pub fn run(args: &Args) -> Result<u8, String> {
    let skill_directory = args.output_directory.join(SKILL_NAME);
    fs::create_dir_all(&skill_directory)
        .map_err(|error| format!("cannot create skill directory {}: {error}", skill_directory.display()))?;

    for (relative_path, content) in SKILLS {
        let skill_path = skill_directory.join(relative_path);
        if let Some(parent) = skill_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create skill directory {}: {error}", parent.display()))?;
        }
        fs::write(&skill_path, content)
            .map_err(|error| format!("cannot write skill {}: {error}", skill_path.display()))?;
        println!("wrote {}", skill_path.display());
    }

    Ok(0)
}
