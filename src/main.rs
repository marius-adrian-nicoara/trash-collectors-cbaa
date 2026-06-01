use quicksilver::{
    geom::{Shape, Rectangle, Vector},
    graphics::{Color, Image, VectorFont, FontRenderer},
    input::Key,
    run, Graphics, Input, Result, Settings, Timer, Window,
};

use rand::distr::{Distribution, Uniform};

pub mod params;
pub mod cbaa_agent;

const WINDOW_SIZE: Vector = Vector{x: 800.0, y: 600.0};
const TEXT_LOCATION: Vector = Vector{x: 20.0, y: 20.0};
const TEXT_SIZE: f32 = 20.0;
const ENTITY_SIZE: Vector = Vector{x: 100.0, y: 100.0};

fn main() {
    run(
        Settings {
            title: "Trash collecting with Consensus-Based Auctioning Algorithm(CBAA)",
            ..Settings::default()
        },
        app,
    );
}

// Our actual logic! Not much to see for this example
async fn app(window: Window, mut gfx: Graphics, mut input: Input) -> Result<()> {
    window.set_size(WINDOW_SIZE);
    // clear window to white
    gfx.clear(Color::WHITE);

    // load images
    let robot_image_path = "vecteezy_cartoon-robot-with-a-big-head-and-a-small-body_63132927_100/vecteezy_cartoon-robot-with-a-big-head-and-a-small-body_63132927.jpg";
    let task_image_path = "vecteezy_trash-bag-vector-hand-drawing_9009212_142/vecteezy_trash-bag-vector-hand-drawing_9009212.jpg";
    let robot_image = Image::load(&gfx, robot_image_path).await?;
    let task_image = Image::load(&gfx, task_image_path).await?;
    
    // load font and initialize FontRenderer
    let ttf = VectorFont::load("font.ttf").await?;
    let mut font = ttf.to_renderer(&gfx, TEXT_SIZE)?;

    /* Generate positions for robots and tasks */
    let mut robots = Vec::new();
    let mut tasks = Vec::new();

    place_entities(params::NR_ROBOTS, params::NR_TASKS, &mut robots, &mut tasks);

    // create agents used for CBAA(Consensus-Based Auctioning Algorithm)
    let mut agent_list = Vec::new();
    for i in 0..params::NR_ROBOTS {
        let agent = cbaa_agent::Agent {
            id: i,
            assignment: params::NO_TASK,
            position: robots[i].pos,
            costs: Vec::new(),
            task_status: [0; params::NR_TASKS],
            bid_list: [f32::NEG_INFINITY; params::NR_TASKS]
        };

        agent_list.push(agent);
    }

    // for each agent, compute cost of attending to each task
    for i in 0..params::NR_ROBOTS {
        agent_list[i].compute_costs(&tasks);
    }

    // establish computation and drawing frequency
    let mut update_timer = Timer::time_per_second(10.0);
    let mut draw_timer = Timer::time_per_second(10.0);

    // keep track of algorithm
    let mut converged: bool = false;
    let mut max_iter_timeout: bool = false;
    let mut iterations: u32 = 0;

    // display initial situation
    draw_entities(&window, &mut gfx, 
                  &mut robots, &robot_image,
                  &mut tasks, &task_image,
                  &agent_list,
                  &iterations, &mut font, 
                  converged, 
                  max_iter_timeout);

    // move to next iteration when Space key is pressed
    // on convergence, press Q key to close window
    loop {
        while let Some(_) = input.next_event().await {};
        
        while update_timer.tick() && !converged && (iterations < params::ITERATIONS_TIMEOUT) {
            // compute next iteration when Space key is pressed
            if input.key_down(Key::Space) {
                // Step 1: place bid for task
                bidding_step(&mut agent_list);

                // Step 2: message neighbors and create consensus for assignments
                converged = consensus_step(&mut agent_list);

                iterations += 1;

                draw_entities(&window, &mut gfx, 
                    &mut robots, &robot_image,
                    &mut tasks, &task_image,
                    &agent_list,
                    &iterations, &mut font, 
                    converged, 
                    max_iter_timeout);
            }
        }

        if converged {
            // close window when Q key is pressed
            if input.key_down(Key::Q) {
                break Ok(())
            }
        }
        else if iterations == params::ITERATIONS_TIMEOUT {
            max_iter_timeout = true;
            // close window when Q key is pressed
            if input.key_down(Key::Q) {
                break Ok(())
            }
        }

        if draw_timer.exhaust().is_some() {
            // draw updates
 
            draw_entities(&window, &mut gfx, 
                          &mut robots, &robot_image,
                          &mut tasks, &task_image,
                          &agent_list,
                          &iterations, &mut font,
                          converged, 
                          max_iter_timeout);
        }
    }
}

fn bidding_step(agent_list: &mut Vec<cbaa_agent::Agent>) {
    for i in 0..params::NR_ROBOTS {
        agent_list[i].get_task();
    }
}

fn consensus_step(agent_list: &mut Vec<cbaa_agent::Agent>) -> bool {
    let mut has_converged: bool;
    let mut convergence_status: Vec<bool> = vec![false; params::NR_ROBOTS];

    for i in 0..params::NR_ROBOTS {
        let mut neighbors: Vec<cbaa_agent::Agent> = Vec::new();

        for j in 0..params::NR_ROBOTS {
            if j != i {
                neighbors.push(agent_list[j].clone());
            }
        }

        convergence_status[i] = agent_list[i].update_task(&mut neighbors);
    }

    has_converged = true;

    for i in 0..params::NR_ROBOTS {
        if convergence_status[i] == false {
            has_converged = false;
        }
    }

    // return
    has_converged
}

// TODO: Return bool status of placement and display status on screen
fn place_entities(nr_robots: usize, nr_tasks: usize,
                  robots: &mut Vec<Rectangle>, tasks: &mut Vec<Rectangle>) {

    let mut rng = rand::rng();
    let mut entities: Vec<Rectangle> = Vec::new();
    let mut overlap: bool = false;

    let mut timeout_counter: u32 = 0;
    
    let x_dist = Uniform::try_from(TEXT_LOCATION.x+TEXT_SIZE+20.0..WINDOW_SIZE.x).unwrap();
    let y_dist = Uniform::try_from(TEXT_LOCATION.y+TEXT_SIZE+20.0..WINDOW_SIZE.y).unwrap();


    while (entities.len() < nr_robots) && (timeout_counter < params::PLACEMENT_TIMEOUT) {

        let current_robot = Rectangle::new(Vector::new(x_dist.sample(&mut rng), y_dist.sample(&mut rng)),
                                           ENTITY_SIZE);

        // check for overlap with all previous robots
        for i in 0..entities.len() {
            overlap = current_robot.overlaps_rectangle(&entities[i]);
            // don't overlap robots
            if overlap {
                break;
            }
        }

        if !overlap {
            robots.push(current_robot);
            entities.push(current_robot);
        }
        timeout_counter += 1;
    }

    if timeout_counter == params::PLACEMENT_TIMEOUT {
        println!("Could not place all robots. Reduce NR_ROBOTS or ENTITY_SIZE");
    }

    while (entities.len() < nr_robots+nr_tasks) && (timeout_counter < params::PLACEMENT_TIMEOUT) {

        let current_task = Rectangle::new(Vector::new(x_dist.sample(&mut rng), y_dist.sample(&mut rng)),
                                          ENTITY_SIZE);

        // check for overlap with all previous robots tasks
        for i in 0..entities.len() {
            overlap = current_task.overlaps_rectangle(&entities[i]);
            // don't overlap with robots or other tasks
            if overlap {
                break;
            }
        }

        if !overlap {
            tasks.push(current_task);
            entities.push(current_task);
        }
        timeout_counter += 1;
    }

    if timeout_counter == params::PLACEMENT_TIMEOUT {
        println!("Could not place all tasks. Reduce NR_ROBOTS, NR_TASKS or ENTITY_SIZE");
    }
}

fn draw_entities(window: &Window, mut gfx: &mut Graphics,
                 robots: &mut Vec<Rectangle>, robot_image: &Image,
                 task_list: &mut Vec<Rectangle>, task_image: &Image,
                 agent_list: &Vec<cbaa_agent::Agent>,
                 iterations: &u32, font: &mut FontRenderer,
                 has_converged: bool,
                 has_timed_out: bool) {
    
    // clear window to white
    gfx.clear(Color::WHITE);

    // how many agents are not currently assigned a task
    let mut ronins: u32 = 0;

    for i in 0..params::NR_ROBOTS {
        gfx.draw_image(&robot_image, robots[i]);
    }

    for i in 0..params::NR_TASKS {
        gfx.draw_image(&task_image, task_list[i]);
    }

    // show connections between agents and tasks
    for i in 0..params::NR_ROBOTS {
        if agent_list[i].assignment != params::NO_TASK {
            // draw line between center of agent and center of task
            let agent_task_pair = vec![agent_list[i].position+ENTITY_SIZE/2.0,
                                       task_list[i].pos+ENTITY_SIZE/2.0];
            gfx.stroke_path(&agent_task_pair, Color::BLACK);
        }
        else {
            ronins +=1; // agent currently has no assignment
        }
    }

    if !has_converged {
        if !has_timed_out {
            let _ = font.draw(
                        &mut gfx,
                        &format!("Iterations: {0}.\nRonins: {1} (agents with no assignment).\nPress Space key to advance.", iterations, ronins),
                        Color::BLACK,
                        TEXT_LOCATION
                    );
        }
        else { // has_time_out = true
            let _ = font.draw(
                        &mut gfx,
                        &format!("Iterations: {0}. Ronins:{1} (agents with no assignment).\nFailed to converge.\nPress Q key to close window.", iterations, ronins),
                        Color::RED,
                        TEXT_LOCATION
                    );
        }
    }
    // has_converged = true
    else {
        let _ = font.draw(
                    &mut gfx,
                    &format!("Iterations: {0}.\nRonins:{1} (agents with no assignment).\nDONE.\nPress Q key to close window.", iterations, ronins),
                    Color::GREEN,
                    TEXT_LOCATION
                );
    }

    let _ = gfx.present(&window);
}