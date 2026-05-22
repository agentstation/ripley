use anyhow::Result;

use ripley_core::dirs;
use ripley_core::rules::RuleSet;
use ripley_core::rules::fetcher;
use ripley_core::rules::registry::SourceRegistry;

use crate::RuleCommands;

pub async fn cmd_rule(command: RuleCommands) -> Result<()> {
    match command {
        RuleCommands::Add { name, url } => cmd_add(&name, &url),
        RuleCommands::Remove { name } => cmd_remove(&name),
        RuleCommands::Update { name } => cmd_update(name.as_deref()).await,
        RuleCommands::List => cmd_list(),
        RuleCommands::Trust { name } => cmd_trust(&name),
        RuleCommands::Untrust { name } => cmd_untrust(&name),
        RuleCommands::Search { query } => cmd_search(&query),
    }
}

fn cmd_add(name: &str, url: &str) -> Result<()> {
    let data_dir = dirs::data_dir()?;
    let mut registry = SourceRegistry::load(&data_dir)?;
    registry.add_source(name.to_string(), url.to_string())?;
    registry.save(&data_dir)?;
    println!("added rule source '{}' ({})", name, url);
    println!("run 'ripley rule update {}' to fetch rules", name);
    Ok(())
}

fn cmd_remove(name: &str) -> Result<()> {
    let data_dir = dirs::data_dir()?;
    let mut registry = SourceRegistry::load(&data_dir)?;
    registry.remove_source(name)?;
    registry.save(&data_dir)?;

    let community_dir = data_dir.join("rules").join("community").join(name);
    if community_dir.exists() {
        std::fs::remove_dir_all(&community_dir)?;
    }

    println!("removed rule source '{}'", name);
    Ok(())
}

async fn cmd_update(name: Option<&str>) -> Result<()> {
    let data_dir = dirs::data_dir()?;
    let mut registry = SourceRegistry::load(&data_dir)?;

    if let Some(source_name) = name {
        let source = registry
            .sources
            .iter_mut()
            .find(|s| s.name == source_name)
            .ok_or_else(|| anyhow::anyhow!("source '{}' not found", source_name))?;
        let count = fetcher::update_source(source, &data_dir).await?;
        println!("updated '{}': {} rules fetched", source_name, count);
    } else {
        if registry.sources.is_empty() {
            println!("no rule sources configured");
            println!("add one with: ripley rule add <name> <url>");
            return Ok(());
        }
        let results = fetcher::update_all_sources(&mut registry, &data_dir).await?;
        for (name, count) in &results {
            println!("  {} — {} rules", name, count);
        }
        if results.is_empty() {
            println!("no sources updated successfully");
        }
    }

    registry.save(&data_dir)?;
    Ok(())
}

fn cmd_list() -> Result<()> {
    let data_dir = dirs::data_dir()?;
    let registry = SourceRegistry::load(&data_dir)?;

    if registry.sources.is_empty() {
        println!("no rule sources configured");
        println!("add one with: ripley rule add <name> <url>");
        return Ok(());
    }

    println!("{:<20} {:<10} {:<8} URL", "NAME", "TRUST", "RULES");
    for source in &registry.sources {
        println!(
            "{:<20} {:<10} {:<8} {}",
            source.name,
            format!("{:?}", source.trust_level).to_lowercase(),
            source.rule_count,
            source.url
        );
    }
    Ok(())
}

fn cmd_trust(name: &str) -> Result<()> {
    let data_dir = dirs::data_dir()?;
    let mut registry = SourceRegistry::load(&data_dir)?;
    registry.trust_source(name)?;
    registry.save(&data_dir)?;
    println!("trusted rule source '{}' — its rules can now block", name);
    Ok(())
}

fn cmd_untrust(name: &str) -> Result<()> {
    let data_dir = dirs::data_dir()?;
    let mut registry = SourceRegistry::load(&data_dir)?;
    registry.untrust_source(name)?;
    registry.save(&data_dir)?;
    println!(
        "untrusted rule source '{}' — its rules will flag but not block",
        name
    );
    Ok(())
}

fn cmd_search(query: &str) -> Result<()> {
    let config_dir = dirs::config_dir()?;
    let data_dir = dirs::data_dir()?;
    let ruleset = RuleSet::load_all(&config_dir, &data_dir)?;

    let query_lower = query.to_lowercase();
    let matches: Vec<_> = ruleset
        .rules()
        .iter()
        .filter(|r| {
            r.id.to_lowercase().contains(&query_lower)
                || r.name.to_lowercase().contains(&query_lower)
                || r.description.to_lowercase().contains(&query_lower)
                || r.signal.to_lowercase().contains(&query_lower)
        })
        .collect();

    if matches.is_empty() {
        println!("no rules matching '{}'", query);
        return Ok(());
    }

    println!("{:<12} {:<30} {:<8} ECOSYSTEM", "ID", "NAME", "WEIGHT");
    for rule in &matches {
        println!(
            "{:<12} {:<30} {:<8} {}",
            rule.id, rule.name, rule.weight, rule.ecosystem
        );
    }
    println!("\n{} rule(s) found", matches.len());
    Ok(())
}
