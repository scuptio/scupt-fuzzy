use arbitrary::{Arbitrary, Unstructured};
use std::collections::HashSet;
use scupt_util::message::Message;
use scupt_util::node_id::NID;
use crate::fuzzy_command::FuzzyCommand;
use crate::fuzzy_event::FuzzyEvent;
use crate::fuzzy_setting::FuzzySetting;

pub trait FuzzyGenerator: Send + Sync {
    fn gen(&self, cmd: FuzzyCommand) -> Vec<FuzzyEvent>;
}


#[derive(Clone)]
pub struct EventGen {
    nodes : Vec<NID>,
    setting:FuzzySetting,
}

impl EventGen {
    pub fn new(
        nodes:Vec<NID>,
        setting:FuzzySetting,
    ) -> Self {
        Self {
            nodes,
            setting
        }
    }

    pub fn fuzz_message(
            &self, message: &Message<String>,
            u:&mut Unstructured,
            vec:&mut Vec<FuzzyEvent>
    ) -> bool {
        let r1 = fuzz_message_event(message, &self.setting, u, vec);
        let r2 = fuzz_crash(message.dest(), &self.setting, u, vec);
        let r3 = fuzz_partition(&self.nodes, &self.setting, u, vec);
        r1.is_ok() && r2.is_ok() && r3.is_ok()
    }
}

fn delayed_message(
    m:&Message<String>,
    setting:&FuzzySetting,
    u:&mut Unstructured
) -> arbitrary::Result<FuzzyEvent> {
    let n = u64::arbitrary(u)?;
    let ms = if setting.message_max_delay_ms == 0 {
        0
    } else {
        n % setting.message_max_delay_ms
    };
    Ok(FuzzyEvent::Delay(ms as u64, m.clone()))
}


fn fuzz_message_event(
    m:&Message<String>,
    setting: &FuzzySetting,
    u:&mut Unstructured,
    output:&mut Vec<FuzzyEvent>
) -> arbitrary::Result<()> {

    let is_delayed = {
        let n = u8::arbitrary(u)?;
        (n as f64 / u8::MAX as f64)  < setting.message_delay_ratio
    };
    if is_delayed {
        let e = delayed_message(m, setting, u)?;
        output.push(e);
    } else {
        output.push(FuzzyEvent::Delay(0, m.clone()));
    }

    let is_repeated = {
        let n = u8::arbitrary(u)?;
        (n as f64 / u8::MAX as f64)  < setting.message_repeat_ratio
    };
    if is_repeated {
        let n = u64::arbitrary(u)?;
        let repeated = if setting.message_max_duplicated == 0 {
            0
        } else {
            n % setting.message_max_duplicated
        };
        for _ in 0..repeated {
            let e = delayed_message(m, setting, u)?;
            output.push(e);
        }
    }
    Ok(())
}

fn fuzz_crash(
    node_id:NID,
    setting: &FuzzySetting,
    u:&mut Unstructured,
    output:&mut Vec<FuzzyEvent>
) -> arbitrary::Result<()> {
    let is_crash = {
        let n = u8::arbitrary(u)?;
        (n as f64 / u8::MAX as f64)  < setting.crash_ratio
    };
    if is_crash {
        let n = if setting.crash_restart_payload.len() == 0 || setting.restart_after_max_ms == 0{
            return Ok(())
        } else {
            let n = u32::arbitrary(u)?;
            n as usize % setting.crash_restart_payload.len()
        };
        let ms = u64::arbitrary(u)?;
        let ms = ms % setting.restart_after_max_ms;
        let (m1, m2) = setting.crash_restart_payload[n].clone();
        output.push(FuzzyEvent::Crash(Message::new(m1, node_id, node_id)));
        output.push(FuzzyEvent::Restart(ms, Message::new(m2, node_id, node_id)));
    }
    Ok(())
}


fn __partition(node_id:&Vec<NID>, n:u64, u:&mut Unstructured) -> arbitrary::Result<(Vec<NID>, Vec<NID>)> {
    let mut nids = node_id.clone();
    nids.sort();
    let mut _vec_i = vec![];
    for i in 0..n {
        let _i = u16::arbitrary(u)?;
        _vec_i.push((i, _i));
    }
    _vec_i.sort_by(|(_, x), (_, y)| {
        x.cmp(y)
    });

    let mut part1 = HashSet::<NID>::new();
    let mut part2 = HashSet::<NID>::new();
    for (_i, _) in _vec_i.iter() {
        let nid = nids[*_i as usize].clone();
        let _ = part1.insert(nid);
    }

    for id in nids.iter() {
        if !part1.contains(id) {
            let _ = part2.insert(*id);
        }
    }

    let p1:Vec<NID> = part1.iter().cloned().collect();
    let p2:Vec<NID> = part2.iter().cloned().collect();
    Ok((p1, p2))
}
fn fuzz_partition(
    node_ids:&Vec<NID>,
    setting: &FuzzySetting,
    u:&mut Unstructured,
    output:&mut Vec<FuzzyEvent>
) -> arbitrary::Result<()> {
    let partition = {
        let n = u8::arbitrary(u)?;
        (n as f64 / u8::MAX as f64)  < setting.network_partition_ratio
    };
    if partition && node_ids.len() > 1 {
        if node_ids.len() == 0 && setting.partition_end_after_max_ms == 0{
            return Ok(())
        }
        let ms = u64::arbitrary(u)?;
        let ms = ms % setting.partition_end_after_max_ms;

        let mut n = u64::arbitrary(u)?;
        n = (n as usize % node_ids.len()) as u64 + 1;
        if n >= node_ids.len() as u64 {
            return Ok(())
        }
        let (p1, p2) = __partition(node_ids, n, u)?;
        output.push(FuzzyEvent::PartitionStart(p1.clone(), p2.clone()));
        output.push(FuzzyEvent::PartitionRecovery(ms, p1.clone(), p2.clone()));
    }
    Ok(())
}
