use 

pub fn set_new_process() {
let _pp: Vec<Process> = database
.conn
.create("process")
.content(Process {
p_id: SUUID::new_v7(),
name: "test".into(),
status: OperataionStatus::New,
create_at: to_u64(UNIX_EPOCH.elapsed().unwrap()),
complete_at: 0,
sla: 0, // TODO Default SLA FROM CONFIG
})
.await?;
}