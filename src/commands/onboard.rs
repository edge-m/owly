use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {}

pub fn run(_args: &Args) -> Result<u8, String> {
    println!(
        "Owly Agent onboarding\n\n\
1. Install the Agent Skill:\n\
   owly skill <agent-skills-directory>\n\n\
2. Make the generated owly/ directory available to your Agent, together\n\
   with the Wiki directory.\n\n\
3. Ask your Agent:\n\
   \"Set up this Wiki with Owly.\"\n\n\
The Agent will use the wiki-config Skill to ask about your Wiki, design\n\
owly.yml interactively, confirm the design, and validate the result.\n\n\
After setup, ask the Agent to:\n\
  - explain the current Wiki structure\n\
  - add an article from a source\n\
  - search the Wiki\n\
  - find maintenance candidates\n\
  - answer questions using the Wiki\n\n\
Useful commands:\n\
  owly init       Create a starter configuration\n\
  owly skill ...  Write the Agent Skills\n\
  owly validate   Validate the Wiki"
    );

    Ok(0)
}
