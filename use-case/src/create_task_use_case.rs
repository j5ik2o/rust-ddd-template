use anyhow::*;
use chrono::Local;
use mopa::*;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use rust_ca_domain::{PostponeableUndoneTask, Task, TaskId, TaskName, TaskRepository, UndoneTask};
use rust_ca_infrastructure::TaskRepositoryInMemory;

#[derive(Debug, Clone)]
pub struct CreateTaskUseCaseCommand {
  id: TaskId,
  name: TaskName,
}

unsafe impl Send for CreateTaskUseCaseCommand {}

impl CreateTaskUseCaseCommand {
  pub fn new(id: TaskId, name: TaskName) -> Self {
    Self { id, name }
  }
}

#[derive(Debug, Clone)]
pub struct CreateTaskUseCaseResult {
  pub id: TaskId,
}

impl CreateTaskUseCaseResult {
  pub fn new(id: TaskId) -> Self {
    Self { id }
  }
}

pub trait CreateTaskUseCase {
  fn execute(&self, request: CreateTaskUseCaseCommand) -> Result<CreateTaskUseCaseResult>;
}

pub struct CreateTaskInteractor {
  task_repository: Arc<Mutex<dyn TaskRepository>>,
}

impl CreateTaskInteractor {
  pub fn new(task_repository: Arc<Mutex<dyn TaskRepository>>) -> Self {
    Self { task_repository }
  }
}

impl CreateTaskUseCase for CreateTaskInteractor {
  fn execute(&self, request: CreateTaskUseCaseCommand) -> Result<CreateTaskUseCaseResult> {
    let id = request.id.clone();
    let name = request.name.clone();
    let task = PostponeableUndoneTask::new(id, name, Local::today() + chrono::Duration::days(1));
    let mut lock = self.task_repository.lock().unwrap();

    // mopaを使ったダウンキャスト
    let task_rc = lock.resolve_by_id(&TaskId(1)).unwrap().unwrap().clone();
    task_rc
      .downcast_ref::<PostponeableUndoneTask>()
      .map(|task| task.postpone());

    lock
      .store(Rc::new(task))
      .map(|_| CreateTaskUseCaseResult::new(request.id.clone()))
  }
}