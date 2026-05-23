use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Duration;
use rand::Rng;

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
    
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => Priority::Critical,
            1 => Priority::High,
            2 => Priority::Medium,
            _ => Priority::Low,
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
    ramp_up_steps: u32,
    ramp_down_steps: u32,
    total_steps: u32,
    current_step: u32,
    total_work_secs: u32,
    ramp_up_secs: u32,
    ramp_down_secs: u32,
    saved_progress: u8,
}

impl Task {
    fn new(id: u64, name: String, priority: Priority, ramp_up_secs: u32, work_secs: u32, ramp_down_secs: u32) -> Self {
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
            total_work_secs: work_secs,
            ramp_up_secs,
            ramp_down_secs,
            saved_progress: 0,
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
            TaskState::Paused => self.saved_progress as f32 / 100.0,
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
            TaskState::Paused => {
                let remaining_steps = self.total_steps - (self.progress as u32 * self.total_steps / 100);
                remaining_steps as f32 / 10.0
            }
            _ => 0.0,
        }
    }
    
    fn format_duration(&self) -> String {
        format!("{}/{}/{}", self.ramp_up_secs, self.total_work_secs, self.ramp_down_secs)
    }
    
    fn pause(&mut self) {
        if self.state == TaskState::RampingUp || self.state == TaskState::Running || self.state == TaskState::RampingDown {
            self.saved_progress = self.progress;
            self.state = TaskState::Paused;
        }
    }
    
    fn resume(&mut self) {
        if self.state == TaskState::Paused {
            self.state = TaskState::Pending;
            self.progress = self.saved_progress;
            self.ramp_up_progress = 0;
            self.ramp_down_progress = 0;
            self.ramping_step = 0;
            self.current_step = (self.progress as u32 * self.total_steps / 100);
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
    
    fn pause_current_task(&mut self) -> Option<Task> {
        if let Some(mut task) = self.heap.pop() {
            if task.state == TaskState::RampingUp || task.state == TaskState::Running || task.state == TaskState::RampingDown {
                task.pause();
                Some(task)
            } else {
                self.heap.push(task);
                None
            }
        } else {
            None
        }
    }
    
    fn resume_task(&mut self, mut task: Task) {
        task.resume();
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
// НАСТРОЙКИ РАНДОМИЗАЦИИ
// ============================================
#[derive(Clone)]
struct RandomizeFlags {
    name: bool,
    priority: bool,
    ramp_up: bool,
    work_time: bool,
    ramp_down: bool,
}

impl RandomizeFlags {
    fn new() -> Self {
        RandomizeFlags {
            name: false,
            priority: false,
            ramp_up: false,
            work_time: false,
            ramp_down: false,
        }
    }
    
    fn all() -> Self {
        RandomizeFlags {
            name: true,
            priority: true,
            ramp_up: true,
            work_time: true,
            ramp_down: true,
        }
    }
}

// ============================================
// ГЕНЕРАТОР СЛУЧАЙНЫХ ЗНАЧЕНИЙ
// ============================================
fn random_task_name() -> String {
    let names = vec![
        "Обновить базу данных",
        "Сделать резервную копию",
        "Отправить отчёт",
        "Проверить логи",
        "Написать документацию",
        "Провести код-ревью",
        "Задеплоить приложение",
        "Запустить тесты",
        "Оптимизировать запросы",
        "Починить баг #42",
        "Настроить CI/CD",
        "Обновить зависимости",
    ];
    let mut rng = rand::thread_rng();
    names[rng.gen_range(0..names.len())].to_string()
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
    randomize_flags: RandomizeFlags,
    keep_dialog_open: bool,
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
            randomize_flags: RandomizeFlags::new(),
            keep_dialog_open: false,
        };
        
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
    
    fn apply_randomization(&mut self) {
        let mut rng = rand::thread_rng();
        
        if self.randomize_flags.name {
            self.new_task_name = random_task_name();
        }
        if self.randomize_flags.priority {
            self.new_task_priority = Priority::random();
        }
        if self.randomize_flags.ramp_up {
            self.new_task_ramp_up = rng.gen_range(0..=5);
        }
        if self.randomize_flags.work_time {
            self.new_task_work_time = rng.gen_range(2..=15);
        }
        if self.randomize_flags.ramp_down {
            self.new_task_ramp_down = rng.gen_range(0..=5);
        }
    }
    
    fn add_current_task(&mut self) {
        if !self.new_task_name.is_empty() {
            self.queue.add_task(
                self.new_task_name.clone(),
                self.new_task_priority.clone(),
                self.new_task_ramp_up,
                self.new_task_work_time,
                self.new_task_ramp_down
            );
            self.status_message = format!("Задача '{}' добавлена", self.new_task_name);
            
            if !self.keep_dialog_open {
                self.new_task_name.clear();
                self.new_task_ramp_up = 1;
                self.new_task_work_time = 5;
                self.new_task_ramp_down = 1;
            } else {
                self.apply_randomization();
            }
        }
    }
}

impl eframe::App for TaskQueueApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_simulation();
        ctx.request_repaint_after(Duration::from_millis(100));
        
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⏸ Пауза симуляции").clicked() {
                    self.simulation_running = false;
                    self.status_message = "Симуляция на паузе".to_string();
                }
                if ui.button("▶ Старт симуляции").clicked() {
                    self.simulation_running = true;
                    self.status_message = "Симуляция запущена".to_string();
                }
                if ui.button("⏸ Приостановить активную задачу").clicked() {
                    if let Some(paused_task) = self.queue.pause_current_task() {
                        self.paused_pool.add(paused_task);
                        self.status_message = "Активная задача приостановлена".to_string();
                    } else {
                        self.status_message = "Нет активной задачи для приостановки".to_string();
                    }
                }
                if ui.button("➕ Новая задача").clicked() {
                    self.show_add_dialog = true;
                    self.keep_dialog_open = false;
                    self.randomize_flags = RandomizeFlags::new();
                }
                if ui.button("🎲 Случайная задача").clicked() {
                    let name = random_task_name();
                    let priority = Priority::random();
                    let mut rng = rand::thread_rng();
                    let ramp_up = rng.gen_range(0..=3);
                    let work_time = rng.gen_range(2..=12);
                    let ramp_down = rng.gen_range(0..=3);
                    self.queue.add_task(name.clone(), priority, ramp_up, work_time, ramp_down);
                    self.status_message = format!("Случайная задача '{}' добавлена", name);
                }
                ui.separator();
                ui.label(format!("Статус: {}", self.status_message));
            });
        });
        
        if self.show_add_dialog {
            egui::Window::new("➕ Новая задача")
                .collapsible(false)
                .resizable(false)
                .default_size([400.0, 450.0])
                .show(ctx, |ui| {
                    ui.heading("Параметры задачи");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("📝 Название:");
                        ui.text_edit_singleline(&mut self.new_task_name);
                        if ui.button("🎲").clicked() {
                            self.new_task_name = random_task_name();
                        }
                    });
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("⚡ Приоритет:");
                        ui.radio_value(&mut self.new_task_priority, Priority::Critical, "Critical");
                        ui.radio_value(&mut self.new_task_priority, Priority::High, "High");
                        ui.radio_value(&mut self.new_task_priority, Priority::Medium, "Medium");
                        ui.radio_value(&mut self.new_task_priority, Priority::Low, "Low");
                        if ui.button("🎲").clicked() {
                            self.new_task_priority = Priority::random();
                        }
                    });
                    
                    ui.separator();
                    
                    ui.label("⏱️ Временные параметры (секунды):");
                    
                    ui.horizontal(|ui| {
                        ui.label("🚀 Запуск (ramp-up):");
                        ui.add(egui::DragValue::new(&mut self.new_task_ramp_up)
                            .clamp_range(0..=10)
                            .speed(0.5));
                        ui.label("сек");
                        if ui.button("🎲").clicked() {
                            let mut rng = rand::thread_rng();
                            self.new_task_ramp_up = rng.gen_range(0..=5);
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("⚙️ Выполнение:");
                        ui.add(egui::DragValue::new(&mut self.new_task_work_time)
                            .clamp_range(1..=30)
                            .speed(0.5));
                        ui.label("сек");
                        if ui.button("🎲").clicked() {
                            let mut rng = rand::thread_rng();
                            self.new_task_work_time = rng.gen_range(2..=15);
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("💾 Завершение (ramp-down):");
                        ui.add(egui::DragValue::new(&mut self.new_task_ramp_down)
                            .clamp_range(0..=10)
                            .speed(0.5));
                        ui.label("сек");
                        if ui.button("🎲").clicked() {
                            let mut rng = rand::thread_rng();
                            self.new_task_ramp_down = rng.gen_range(0..=5);
                        }
                    });
                    
                    ui.separator();
                    
                    ui.label(format!("📊 Итого: {} сек", 
                        self.new_task_ramp_up + self.new_task_work_time + self.new_task_ramp_down));
                    
                    ui.separator();
                    
                    ui.collapsing("🎲 Настройки рандомизации", |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Всё случайно").clicked() {
                                self.randomize_flags = RandomizeFlags::all();
                                self.apply_randomization();
                            }
                            if ui.button("Сбросить").clicked() {
                                self.randomize_flags = RandomizeFlags::new();
                            }
                        });
                        
                        ui.checkbox(&mut self.randomize_flags.name, "Случайное название");
                        ui.checkbox(&mut self.randomize_flags.priority, "Случайный приоритет");
                        ui.checkbox(&mut self.randomize_flags.ramp_up, "Случайный запуск");
                        ui.checkbox(&mut self.randomize_flags.work_time, "Случайное время работы");
                        ui.checkbox(&mut self.randomize_flags.ramp_down, "Случайное завершение");
                        
                        if ui.button("Применить рандомизацию").clicked() {
                            self.apply_randomization();
                        }
                    });
                    
                    ui.checkbox(&mut self.keep_dialog_open, "Оставлять окно открытым");
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("✅ Добавить").clicked() {
                            self.add_current_task();
                            if !self.keep_dialog_open {
                                self.show_add_dialog = false;
                            }
                        }
                        if ui.button("➕ Добавить и очистить").clicked() {
                            self.add_current_task();
                            self.new_task_name.clear();
                            self.new_task_ramp_up = 1;
                            self.new_task_work_time = 5;
                            self.new_task_ramp_down = 1;
                        }
                        if ui.button("❌ Закрыть").clicked() {
                            self.show_add_dialog = false;
                        }
                    });
                });
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 Симулятор очереди с приоритетом");
            ui.separator();
            
            ui.collapsing("📌 Активная задача", |ui| {
                let current_task_info = self.queue.current_task().map(|task| {
                    (task.name.clone(), task.priority.clone(), task.state.clone(), 
                     task.display_progress(), task.ramp_up_progress, task.progress, 
                     task.ramp_down_progress, task.get_remaining_time_secs(),
                     task.format_duration())
                });
                
                if let Some((name, priority, state, progress, ramp_up, running_progress, ramp_down, remaining_time, duration)) = current_task_info {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", state.emoji(), name));
                        ui.colored_label(priority.color(), priority.to_string());
                        ui.label(format!("[{}]", duration));
                    });
                    ui.label(format!("Состояние: {}", state.to_string()));
                    ui.label(format!("Осталось: {:.1} сек", remaining_time));
                    
                    let progress_bar = egui::ProgressBar::new(progress)
                        .desired_width(400.0)
                        .show_percentage();
                    ui.add(progress_bar);
                    
                    match state {
                        TaskState::RampingUp => {
                            ui.label(format!("Прогресс запуска: {}%", ramp_up));
                        }
                        TaskState::Running => {
                            ui.label(format!("Выполнено: {}%", running_progress));
                        }
                        TaskState::RampingDown => {
                            ui.label(format!("Прогресс завершения: {}%", ramp_down));
                        }
                        _ => {}
                    }
                } else {
                    ui.label("Нет активных задач");
                }
            });
            
            ui.separator();
            
            ui.collapsing("📋 Очередь задач", |ui| {
                let tasks = self.queue.get_all_tasks();
                if tasks.is_empty() {
                    ui.label("Очередь пуста");
                } else {
                    ui.columns(5, |columns| {
                        columns[0].label("Состояние");
                        columns[1].label("Приоритет");
                        columns[2].label("Название");
                        columns[3].label("Время");
                        columns[4].label("Прогресс");
                    });
                    for task in tasks {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}", task.state.emoji()));
                            ui.colored_label(task.priority.color(), format!("[{}]", task.priority.to_string()));
                            ui.label(&task.name);
                            ui.label(task.format_duration());
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
            
            ui.collapsing("⏸ Приостановленные задачи", |ui| {
                let paused_items: Vec<(u64, String, Priority, u8, f32, String)> = self.paused_pool
                    .get_all()
                    .iter()
                    .map(|task| (task.id, task.name.clone(), task.priority.clone(), task.progress, 
                                 task.get_remaining_time_secs(), task.format_duration()))
                    .collect();
                
                if paused_items.is_empty() {
                    ui.label("Нет приостановленных задач");
                } else {
                    ui.columns(5, |columns| {
                        columns[0].label("Состояние");
                        columns[1].label("Приоритет");
                        columns[2].label("Название");
                        columns[3].label("Время");
                        columns[4].label("Инфо");
                    });
                    for (id, name, priority, progress, remaining, duration) in paused_items {
                        ui.horizontal(|ui| {
                            ui.label("⏸");
                            ui.colored_label(priority.color(), format!("[{}]", priority.to_string()));
                            ui.label(&name);
                            ui.label(duration);
                            ui.label(format!("{}%, осталось {:.1}с", progress, remaining));
                            
                            if ui.button("▶").clicked() {
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
                "ℹ️ Приоритеты: Critical > High > Medium > Low | 1 шаг = 100 мс | При паузе запуск/завершение сбрасываются");
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([950.0, 750.0])
            .with_title("Очередь с приоритетом - Rust Lab"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Task Queue Simulator",
        options,
        Box::new(|_cc| Box::new(TaskQueueApp::new())),
    )
}