use std::collections::HashMap;

use crate::{
    cache_handling::{Overview, ScoreAvg},
    data_base::{self, DataBase},
    stat_sums::StatSumFile,
    tags::TagList,
};

#[derive(Default)]
struct Scores {
    m_score: ScoreAvg,
    p_score: ScoreAvg,
}

#[derive(Default)]
struct ScoreCollection {
    count: u32,
    m_max: u32,
    m_min: u32,
    m_avg: f32,
    p_max: u32,
    p_min: u32,
    p_avg: f32,
}

impl ScoreCollection {
    fn add(&mut self, scores: &Overview) {
        self.count += 1;
        self.m_max += scores.m_score.max as u32;
        self.m_min += scores.m_score.min as u32;
        self.m_avg += scores.m_score.avg();

        self.p_max += scores.p_score.max as u32;
        self.p_min += scores.p_score.min as u32;
        self.p_avg += scores.p_score.avg();
    }

    fn to_avg(&self) -> [f32; 6] {
        let mut values = [0.0; 6];
        values[0] = self.m_max as f32 / self.count as f32;
        values[1] = self.m_min as f32 / self.count as f32;
        values[2] = self.m_avg / self.count as f32;
        values[3] = self.p_max as f32 / self.count as f32;
        values[4] = self.p_min as f32 / self.count as f32;
        values[5] = self.p_avg / self.count as f32;

        values
    }
}

pub fn get_tag_scores(data_base: &DataBase) -> Result<(), data_base::Error> {
    let mut global_scores = Scores::default();
    let mut score_collection = ScoreCollection::default();
    let mut tag_scores: HashMap<u16, ScoreCollection> = HashMap::new();

    for overview in data_base.data_files()?.iter().map(|df| df.get_overview()) {
        for tag in &overview.tags {
            let tagscore = tag_scores.entry(*tag).or_default();
            //tagscore.p_score.merge(&overview.p_score);
            //tagscore.m_score.merge(&overview.m_score);

            tagscore.add(&overview);

            global_scores.m_score.merge(&overview.m_score);
            global_scores.p_score.merge(&overview.p_score);
            score_collection.add(&overview);
        }
    } // */
      /*
      for entries in data_base.data_files()?.iter().map(|df| df.entries()) {
          for entry in entries.values() {
              for tag in &entry.tags {
                  let tagscore = tag_scores.entry(*tag).or_default();
                  tagscore.m_score.add(entry.mental_score);
                  tagscore.p_score.add(entry.physical_score);

                  global_scores.m_score.add(entry.mental_score);
                  global_scores.p_score.add(entry.physical_score);
                  score_collection.add(&overview);
              }
          }
      } // */

    let (avg_m_score, avg_p_score) = (global_scores.m_score.avg(), global_scores.p_score.avg());

    let averages = score_collection.to_avg();

    let tag_scores = tag_scores.iter().collect::<Vec<_>>();
    let mut by_m_scores = tag_scores
        .clone()
        .iter()
        .map(|ts| {
            let avgs = ts.1.to_avg();
            (ts.0, avgs[1] - averages[1])
        })
        .collect::<Vec<_>>();
    by_m_scores.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut by_p_scores = tag_scores
        .clone()
        .iter()
        .map(|ts| {
            let avgs = ts.1.to_avg();
            (ts.0, avgs[4] - averages[4])
        })
        //.map(|ts| (ts.0, ts.1.p_score.min as f32 - averages[4]))
        .collect::<Vec<_>>();
    by_p_scores.sort_by(|a, b| b.1.total_cmp(&a.1));

    let tags = TagList::from_file(data_base.path())?;

    let sums = StatSumFile::load(&data_base.path().stat_sums().join("global_sums.txt"))?;

    println!(
        "avg_m_score: {}, avg_p_score: {}\nM: Max, Min, Avg | P: Max, Min, Avg\n{:?}\n",
        avg_m_score, avg_p_score, averages
    );

    for (tag, score) in by_p_scores {
        println!(
            "{}:{}, Score: {}",
            tags.get_tag(*tag)?,
            sums.tags().get_occurances(*tag),
            score
        );
    }

    todo!();
}
