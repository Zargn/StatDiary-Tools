use std::collections::HashMap;

use crate::{
    cache_handling::ScoreAvg,
    data_base::{self, DataBase},
    tags::TagList,
};

#[derive(Default)]
struct Scores {
    m_score: ScoreAvg,
    p_score: ScoreAvg,
}

pub fn get_tag_scores(data_base: &DataBase) -> Result<(), data_base::Error> {
    let mut global_scores = Scores::default();
    let mut tag_scores: HashMap<u16, Scores> = HashMap::new();
    /*
    for overview in data_base.data_files()?.iter().map(|df| df.get_overview()) {
        for tag in overview.tags {
            let tagscore = tag_scores.entry(tag).or_default();
            tagscore.p_score.merge(&overview.p_score);
            tagscore.m_score.merge(&overview.m_score);

            global_scores.m_score.merge(&overview.m_score);
            global_scores.p_score.merge(&overview.p_score);
        }
    } // */

    for entries in data_base.data_files()?.iter().map(|df| df.entries()) {
        for entry in entries.values() {
            for tag in &entry.tags {
                let tagscore = tag_scores.entry(*tag).or_default();
                tagscore.m_score.add(entry.mental_score);
                tagscore.p_score.add(entry.physical_score);

                global_scores.m_score.add(entry.mental_score);
                global_scores.p_score.add(entry.physical_score);
            }
        }
    }

    let (avg_m_score, avg_p_score) = (global_scores.m_score.avg(), global_scores.p_score.avg());

    let tag_scores = tag_scores.iter().collect::<Vec<_>>();
    let mut by_m_scores = tag_scores
        .clone()
        .iter()
        .map(|ts| (ts.0, ts.1.m_score.avg() - avg_m_score))
        .collect::<Vec<_>>();
    by_m_scores.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut by_p_scores = tag_scores
        .clone()
        .iter()
        .map(|ts| (ts.0, ts.1.p_score.avg() - avg_p_score))
        .collect::<Vec<_>>();
    by_p_scores.sort_by(|a, b| b.1.total_cmp(&a.1));

    let tags = TagList::from_file(&data_base.path())?;
    println!("avg_m_score: {}, avg_p_score: {}", avg_m_score, avg_p_score);

    for (tag, score) in by_p_scores {
        println!("Tag: {}, Score: {}", tags.get_tag(*tag)?, score);
    }

    todo!();
}
