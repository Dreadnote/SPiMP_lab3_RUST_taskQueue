use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Duration;

// ============================================
// ПРИОРИТЕТЫ
// ============================================
#[derive(Debug, Clone, PartialEq, Eq)]
enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

impl Priority {
    fn color(&self) -> egui::Color32 {
        match self {
            Priority::Critical => egui::Color32::from_rgb(220, 20, 60),
            Priority::High => egui::Color32::from_rgb(255, 140, 0),
            Priority::Medium => egui::Color32::from_rgb(255, 215, 0),
            Priority::Low => egui::Color32::from_rgb(50, 205, 50),
        }
    }
    
    fn to_string(&self) -> &'static str {
        match self {
            Priority::Critical => "CRITICAL",
            Priority::High => "HIGH",
            Priority::Medium => "MEDIUM",
            Priority::Low => "LOW",
        }
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Priority::Critical, Priority::Critical) => Ordering::Equal,
            (Priority::Critical, _) => Ordering::Greater,
            (_, Priority::Critical) => Ordering::Less,
            (Priority::High, Priority::High) => Ordering::Equal,
            (Priority::High, _) => Ordering::Greater,
            (_, Priority::High) => Ordering::Less,
            (Priority::Medium, Priority::Medium) => Ordering::Equal,
            (Priority::Medium, _) => Ordering::Greater,
            (_, Priority::Medium) => Ordering::Less,
            (Priority::Low, Priority::Low) => Ordering::Equal,
        }
    }
}

// ============================================
// СОСТОЯНИЯ ЗАДАЧИ
// ============================================
#[derive(Debug, Clone, PartialEq)]
enum TaskState {
    Pending,
    RampingUp,
    Running,
    RampingDown,
    Completed,
    Paused,
}

impl TaskState {
    fn emoji(&self) -> &'static str {
        match self {
            TaskState::Pending => "⏳",
            TaskState::RampingUp => "🚀",
            TaskState::Running => "▶",
            TaskState::RampingDown => "💾",
            TaskState::Completed => "✅",
            TaskState::Paused => "⏸",
        }
    }
    
    fn to_string(&self) -> &'static str {
        match self {
            TaskState::Pending => "Ожидание",
            TaskState::RampingUp => "Запуск",
            TaskState::Running => "Выполнение",
            TaskState::RampingDown => "Завершение",
            TaskState::Completed => "Готово",
            TaskState::Paused => "Пауза",
        }
    }
}

// ============================================
// СТРУКТУРА ЗАДАЧИ
// ============================================
#[derive(Debug, Clone)]
struct Task {
    id: u64,
    name: String,
    priority: Priority,
    progress: u8,
    state: TaskState,
    ramp_up_progress: u8,
    ramp_down_progress: u8,
    ramping_step: u32,
    // Новые поля для времени
    ramp_up_steps: u32,    // сколько шагов на запуск
    ramp_down_steps: u32,  // сколько шагов на завершение
    total_steps: u32,      // сколько всего шагов выполнения
    current_step: u32,     // текущий шаг выполнения
}

impl Task {
    fn new(id: u64, name: String, priority: Priority, ramp_up_secs: u32, work_secs: u32, ramp_down_secs: u32) -> Self {
        // Переводим секунды в шаги (1 шаг = 100 мс)
        let ramp_up_steps = ramp_up_secs * 10;
        let total_steps = work_secs * 10;
        let ramp_down_steps = ramp_down_secs * 10;
        
        Task {
            id,
            name,
            priority,
            progress: 0,
            state: TaskState::Pending,
            ramp_up_progress: 0,
            ramp_down_progress: 0,
            ramping_step: 0,
            ramp_up_steps,
            ramp_down_steps,
            total_steps,
            current_step: 0,
        }
    }
    
    fn step(&mut self) -> bool {
        match self.state {
            TaskState::Pending => {
                if self.ramp_up_steps > 0 {
                    self.state = TaskState::RampingUp;
                    self.ramping_step = 0;
                    self.ramp_up_progress = 0;
                } else {
                    // Если нет времени на запуск, сразу в выполнение
                    self.state = TaskState::Running;
                    self.current_step = 0;
                }
            }
            TaskState::RampingUp => {
                self.ramping_step += 1;
                self.ramp_up_progress = (self.ramping_step * 100 / self.ramp_up_steps) as u8;
                if self.ramping_step >= self.ramp_up_steps {
                    self.state = TaskState::Running;
                    self.ramp_up_progress = 100;
                    self.current_step = 0;
                }
            }
            TaskState::Running => {
                self.current_step += 1;
                self.progress = (self.current_step * 100 / self.total_steps) as u8;
                if self.current_step >= self.total_steps {
                    self.progress = 100;
                    if self.ramp_down_steps > 0 {
                        self.state = TaskState::RampingDown;
                        self.ramping_step = 0;
                        self.ramp_down_progress = 0;
                    } else {
                        self.state = TaskState::Completed;
                        return true;
                    }
                }
            }
            TaskState::RampingDown => {
                self.ramping_step += 1;
                self.ramp_down_progress = (self.ramping_step * 100 / self.ramp_down_steps) as u8;
                if self.ramping_step >= self.ramp_down_steps {
                    self.state = TaskState::Completed;
                    return true;
                }
            }
            _ => {}
        }
        false
    }
    
    fn display_progress(&self) -> f32 {
        match self.state {
            TaskState::RampingUp => self.ramp_up_progress as f32 / 100.0,
            TaskState::Running => self.progress as f32 / 100.0,
            TaskState::RampingDown => 1.0 - (self.ramp_down_progress as f32 / 100.0),
            TaskState::Completed => 1.0,
            _ => 0.0,
        }
    }
    
    fn get_remaining_time_secs(&self) -> f32 {
        match self.state {
            TaskState::RampingUp => {
                let remaining_steps = self.ramp_up_steps - self.ramping_step;
                remaining_steps as f32 / 10.0
            }
            TaskState::Running => {
                let remaining_steps = self.total_steps - self.current_step;
                remaining_steps as f32 / 10.0
            }
            TaskState::RampingDown => {
                let remaining_steps = self.ramp_down_steps - self.ramping_step;
                remaining_steps as f32 / 10.0
            }
            _ => 0.0,
        }
    }
}

impl Eq for Task {}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => other.id.cmp(&self.id),
            other => other,
        }
    }
}

// ============================================
// ПРИОРИТЕТНАЯ ОЧЕРЕДЬ
// ============================================
struct PriorityQueue {
    heap: BinaryHeap<Task>,
    next_id: u64,
}

impl PriorityQueue {
    fn new() -> Self {
        PriorityQueue {
            heap: BinaryHeap::new(),
            next_id: 1,
        }
    }
    
    fn add_task(&mut self, name: String, priority: Priority, ramp_up_secs: u32, work_secs: u32, ramp_down_secs: u32) {
        let task = Task::new(self.next_id, name, priority, ramp_up_secs, work_secs, ramp_down_secs);
        self.next_id += 1;
        self.heap.push(task);
    }
    
    fn step_current(&mut self) -> bool {
        if let Some(mut task) = self.heap.pop() {
            let completed = task.step();
            if !completed && task.state != TaskState::Completed {
                self.heap.push(task);
                false
            } else {
                true
            }
        } else {
            false
        }
    }
    
    fn current_task(&self) -> Option<&Task> {
        self.heap.peek()
    }
    
    fn pause_task(&mut self, id: u64) -> Option<Task> {
        let mut temp = Vec::new();
        let mut found = None;
        
        while let Some(task) = self.heap.pop() {
            if task.id == id && task.state == TaskState::Running {
                let mut task = task;
                task.state = TaskState::Paused;
                found = Some(task);
                break;
            } else {
                temp.push(task);
            }
        }
        
        for task in temp {
            self.heap.push(task);
        }
        
        found
    }
    
    fn resume_task(&mut self, mut task: Task) {
        task.state = TaskState::Pending;
        self.heap.push(task);
    }
    
    fn get_all_tasks(&self) -> Vec<Task> {
        let mut tasks: Vec<Task> = self.heap.iter().cloned().collect();
        tasks.sort_by(|a, b| b.cmp(a));
        tasks
    }
}

// ============================================
// ПУЛЛ ПРИОСТАНОВЛЕННЫХ
// ============================================
struct PausedPool {
    tasks: Vec<Task>,
}

impl PausedPool {
    fn new() -> Self {
        PausedPool { tasks: Vec::new() }
    }
    
    fn add(&mut self, task: Task) {
        self.tasks.push(task);
    }
    
    fn remove(&mut self, id: u64) -> Option<Task> {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
            Some(self.tasks.remove(pos))
        } else {
            None
        }
    }
    
    fn get_all(&self) -> &[Task] {
        &self.tasks
    }
}

// ============================================
// GUI ПРИЛОЖЕНИЕ
// ============================================
struct TaskQueueApp {
    queue: PriorityQueue,
    paused_pool: PausedPool,
    simulation_running: bool,
    new_task_name: String,
    new_task_priority: Priority,
    new_task_ramp_up: u32,
    new_task_work_time: u32,
    new_task_ramp_down: u32,
    show_add_dialog: bool,
    status_message: String,
}

impl TaskQueueApp {
    fn new() -> Self {
        let mut app = TaskQueueApp {
            queue: PriorityQueue::new(),
            paused_pool: PausedPool::new(),
            simulation_running: true,
            new_task_name: String::new(),
            new_task_priority: Priority::Medium,
            new_task_ramp_up: 1,
            new_task_work_time: 5,
            new_task_ramp_down: 1,
            show_add_dialog: false,
            status_message: "Готов к работе".to_string(),
        };
        
        // Добавляем тестовые задачи с разными временами
        app.queue.add_task("Отрендерить видео".to_string(), Priority::High, 2, 8, 1);
        app.queue.add_task("Ответить на письма".to_string(), Priority::Medium, 1, 3, 1);
        app.queue.add_task("Сделать бэкап".to_string(), Priority::Low, 3, 10, 2);
        app.queue.add_task("Срочный баг".to_string(), Priority::Critical, 1, 2, 1);
        
        app
    }
    
    fn update_simulation(&mut self) {
        if self.simulation_running {
            let _completed = self.queue.step_current();
        }
    }
}

impl eframe::App for TaskQueueApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_simulation();
        ctx.request_repaint_after(Duration::from_millis(100));
        
        // Верхнее меню
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⏸ Пауза").clicked() {
                    self.simulation_running = false;
                    self.status_message = "Симуляция на паузе".to_string();
                }
                if ui.button("▶ Старт").clicked() {
                    self.simulation_running = true;
                    self.status_message = "Симуляция запущена".to_string();
                }
                if ui.button("➕ Новая задача").clicked() {
                    self.show_add_dialog = true;
                }
                ui.separator();
                ui.label(format!("Статус: {}", self.status_message));
            });
        });
        
        // Диалог добавления задачи
        if self.show_add_dialog {
            egui::Window::new("➕ Новая задача")
                .collapsible(false)
                .resizable(false)
                .default_size([350.0, 280.0])
                .show(ctx, |ui| {
                    ui.heading("Параметры задачи");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("📝 Название:");
                        ui.text_edit_singleline(&mut self.new_task_name);
                    });
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("⚡ Приоритет:");
                        ui.radio_value(&mut self.new_task_priority, Priority::Critical, "Critical");
                        ui.radio_value(&mut self.new_task_priority, Priority::High, "High");
                        ui.radio_value(&mut self.new_task_priority, Priority::Medium, "Medium");
                        ui.radio_value(&mut self.new_task_priority, Priority::Low, "Low");
                    });
                    
                    ui.separator();
                    
                    ui.label("⏱️ Временные параметры (секунды):");
                    
                    ui.horizontal(|ui| {
                        ui.label("🚀 Запуск (ramp-up):");
                        ui.add(egui::DragValue::new(&mut self.new_task_ramp_up)
                            .clamp_range(0..=10)
                            .speed(0.5));
                        ui.label("сек");
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("⚙️ Выполнение:");
                        ui.add(egui::DragValue::new(&mut self.new_task_work_time)
                            .clamp_range(1..=30)
                            .speed(0.5));
                        ui.label("сек");
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("💾 Завершение (ramp-down):");
                        ui.add(egui::DragValue::new(&mut self.new_task_ramp_down)
                            .clamp_range(0..=10)
                            .speed(0.5));
                        ui.label("сек");
                    });
                    
                    ui.separator();
                    
                    ui.label(format!("📊 Итого: {} сек (запуск: {} + работа: {} + завершение: {})",
                        self.new_task_ramp_up + self.new_task_work_time + self.new_task_ramp_down,
                        self.new_task_ramp_up,
                        self.new_task_work_time,
                        self.new_task_ramp_down));
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("✅ Добавить").clicked() {
                            if !self.new_task_name.is_empty() {
                                self.queue.add_task(
                                    self.new_task_name.clone(),
                                    self.new_task_priority.clone(),
                                    self.new_task_ramp_up,
                                    self.new_task_work_time,
                                    self.new_task_ramp_down
                                );
                                self.status_message = format!("Задача '{}' добавлена", self.new_task_name);
                                self.new_task_name.clear();
                                self.new_task_ramp_up = 1;
                                self.new_task_work_time = 5;
                                self.new_task_ramp_down = 1;
                                self.show_add_dialog = false;
                            }
                        }
                        if ui.button("❌ Отмена").clicked() {
                            self.show_add_dialog = false;
                        }
                    });
                });
        }
        
        // Основная панель
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 Симулятор очереди с приоритетом");
            ui.separator();
            
            // Текущая активная задача
            ui.collapsing("📌 Активная задача", |ui| {
                let current_task_info = self.queue.current_task().map(|task| {
                    (task.id, task.name.clone(), task.priority.clone(), task.state.clone(), 
                     task.display_progress(), task.ramp_up_progress, task.progress, 
                     task.ramp_down_progress, task.get_remaining_time_secs())
                });
                
                if let Some((id, name, priority, state, progress, ramp_up, running_progress, ramp_down, remaining_time)) = current_task_info {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", state.emoji(), name));
                        ui.colored_label(priority.color(), priority.to_string());
                    });
                    ui.label(format!("Состояние: {}", state.to_string()));
                    ui.label(format!("⏱️ Осталось: {:.1} сек", remaining_time));
                    
                    let progress_bar = egui::ProgressBar::new(progress)
                        .desired_width(400.0)
                        .show_percentage();
                    ui.add(progress_bar);
                    
                    match state {
                        TaskState::RampingUp => {
                            ui.label(format!("🚀 Прогресс запуска: {}%", ramp_up));
                        }
                        TaskState::Running => {
                            ui.label(format!("⚙️ Выполнено: {}%", running_progress));
                        }
                        TaskState::RampingDown => {
                            ui.label(format!("💾 Прогресс завершения: {}%", ramp_down));
                        }
                        _ => {}
                    }
                    
                    if ui.button("⏸ Приостановить эту задачу").clicked() {
                        if let Some(paused_task) = self.queue.pause_task(id) {
                            self.paused_pool.add(paused_task);
                            self.status_message = format!("Задача {} приостановлена", name);
                        }
                    }
                } else {
                    ui.label("Нет активных задач");
                }
            });
            
            ui.separator();
            
            // Очередь задач
            ui.collapsing("📋 Очередь задач", |ui| {
                let tasks = self.queue.get_all_tasks();
                if tasks.is_empty() {
                    ui.label("Очередь пуста");
                } else {
                    ui.columns(4, |columns| {
                        columns[0].label("Состояние");
                        columns[1].label("Приоритет");
                        columns[2].label("Название");
                        columns[3].label("Прогресс");
                    });
                    for task in tasks {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}", task.state.emoji()));
                            ui.colored_label(task.priority.color(), format!("[{}]", task.priority.to_string()));
                            ui.label(&task.name);
                            if task.progress > 0 {
                                ui.label(format!("{}%", task.progress));
                            } else if task.ramp_up_progress > 0 {
                                ui.label(format!("запуск {}%", task.ramp_up_progress));
                            } else {
                                ui.label("ожидает");
                            }
                        });
                    }
                }
            });
            
            ui.separator();
            
            // Приостановленные задачи
            ui.collapsing("⏸ Приостановленные задачи", |ui| {
                let paused_items: Vec<(u64, String, Priority, u8, f32)> = self.paused_pool
                    .get_all()
                    .iter()
                    .map(|task| (task.id, task.name.clone(), task.priority.clone(), task.progress, task.get_remaining_time_secs()))
                    .collect();
                
                if paused_items.is_empty() {
                    ui.label("Нет приостановленных задач");
                } else {
                    for (id, name, priority, progress, remaining) in paused_items {
                        ui.horizontal(|ui| {
                            ui.label("⏸");
                            ui.colored_label(priority.color(), format!("[{}]", priority.to_string()));
                            ui.label(&name);
                            ui.label(format!("Прогресс: {}%", progress));
                            ui.label(format!("⏱️ осталось: {:.1}с", remaining));
                            
                            if ui.button("▶ Возобновить").clicked() {
                                if let Some(resumed_task) = self.paused_pool.remove(id) {
                                    self.queue.resume_task(resumed_task);
                                    self.status_message = format!("Задача {} возобновлена", name);
                                }
                            }
                        });
                    }
                }
            });
            
            ui.separator();
            
            ui.colored_label(egui::Color32::from_rgb(100, 100, 100), 
                "ℹ️ Управление: кнопки вверху | Приоритеты: Critical > High > Medium > Low | 1 шаг = 100 мс");
        });
    }
}

// ============================================
// ГЛАВНАЯ ФУНКЦИЯ
// ============================================
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("Очередь с приоритетом - Rust Lab"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Task Queue Simulator",
        options,
        Box::new(|_cc| Box::new(TaskQueueApp::new())),
    )
}