use anyhow::Result;
use chrono::prelude::*;
use octocrab;
use octocrab::params;
use octocrab::Octocrab;
use std::collections::HashMap;
use tabled::Table;
use tabled::Tabled;

#[derive(Tabled)]
struct AuthorOpenPRs {
    name: String,
    open: u32,
}

#[derive(Tabled)]
struct PRDuration {
    name: String,
    duration: i64,
}

async fn print_lifetime() -> Result<()> {
    let octocrab = Octocrab::default();

    let mut current_page = octocrab
        .pulls("llvm", "llvm-project")
        .list()
        .state(params::State::Closed)
        .head("main")
        .per_page(100)
        .send()
        .await?;

    let mut prs = current_page.take_items();

    let mut lifetimes: Vec<PRDuration> = Vec::with_capacity(100);

    while let Ok(Some(mut new_page)) = octocrab.get_page(&current_page.next).await {
        prs.extend(new_page.take_items());

        for pr in prs.drain(..) {
            if let Some(closed_at) = pr.closed_at {
                let created_at: DateTime<Utc> = pr.created_at.unwrap();
                let diff = closed_at.signed_duration_since(created_at);
                lifetimes.push(PRDuration {
                    name: pr.user.unwrap().login,
                    duration: diff.num_hours(),
                });
            }
        }

        current_page = new_page;
    }

    lifetimes.sort_by(|a, b| a.duration.cmp(&b.duration));
    lifetimes.reverse();
    lifetimes.truncate(10);

    println!("Lifetime of PRs");

    println!("{}", Table::new(lifetimes).to_string());

    Ok(())
}

async fn print_top_authors() -> Result<()> {
    let octocrab = Octocrab::default();

    let mut current_page = octocrab
        .pulls("llvm", "llvm-project")
        .list()
        .state(params::State::Open)
        .head("main")
        .per_page(100)
        .send()
        .await?;

    let mut prs = current_page.take_items();

    let mut authors_frequency: HashMap<String, u32> = HashMap::new();
    while let Ok(Some(mut new_page)) = octocrab.get_page(&current_page.next).await {
        prs.extend(new_page.take_items());

        for pr in prs.drain(..) {
            if let Some(author) = pr.user {
                authors_frequency
                    .entry(author.login)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }

        current_page = new_page;
    }

    let mut authors: Vec<AuthorOpenPRs> = Vec::with_capacity(authors_frequency.len());

    for (author, open) in &authors_frequency {
        authors.push(AuthorOpenPRs {
            name: author.to_string(),
            open: *open,
        });
    }

    authors.sort_by(|a, b| a.open.cmp(&b.open));
    authors.reverse();
    authors.truncate(10);

    println!("Authors and open PRs");

    println!("{}", Table::new(authors).to_string());

    //for index in 1..10 {
    //    let entry = &authors[index];
    //    println!("{0}          : {1}", entry.name, entry.open);
    //}

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    print_top_authors().await?;

    print_lifetime().await?;

    Ok(())
}
