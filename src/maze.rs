use std::fs::File;
use std::io::{BufRead, BufReader};
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;

pub type Maze = Vec<Vec<char>>;

/// Carga un laberinto desde un fichero de texto, cada línea
/// es un `Vec<char>`.
pub fn load_maze(filename: &str) -> Maze {
    let file   = File::open(filename).expect("No se pudo abrir el archivo");
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| l.expect("Error leyendo línea").chars().collect())
        .collect()
}

/// Genera un laberinto procedural con DFS en un grid de celdas.
/// El resultado es una malla de caracteres donde:
/// - '#' son muros  
/// - ' ' (espacio) suelo  
/// - 'p' posición inicial del jugador  
/// - 'g' meta
pub fn make_maze(cell_w: usize, cell_h: usize) -> Maze {
    let gw = cell_w * 2 + 1;
    let gh = cell_h * 2 + 1;

    // Inicializa todo como muros
    let mut maze    = vec![vec!['#'; gw]; gh];
    let mut visited = vec![vec![false; cell_w]; cell_h];
    let mut rng     = thread_rng();

    // Recursión DFS para “carve”
    fn carve(
        cx: usize,
        cy: usize,
        cw: usize,
        ch: usize,
        maze: &mut Maze,
        visited: &mut Vec<Vec<bool>>,
        rng: &mut impl Rng,
    ) {
        visited[cy][cx] = true;

        let mut dirs = vec![(0isize, -1isize), (1, 0), (0, 1), (-1, 0)];
        dirs.shuffle(rng);

        for &(dx, dy) in &dirs {
            let nx = cx as isize + dx;
            let ny = cy as isize + dy;
            if nx < 0 || nx >= cw as isize || ny < 0 || ny >= ch as isize {
                continue;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            if visited[ny][nx] {
                continue;
            }

            // Coordenadas en el grid de chars
            let x1 = cx * 2 + 1;
            let y1 = cy * 2 + 1;
            let x2 = nx * 2 + 1;
            let y2 = ny * 2 + 1;

            // Destruye los muros interiores
            maze[y1][x1] = ' ';
            maze[y2][x2] = ' ';
            maze[(y1 + y2) / 2][(x1 + x2) / 2] = ' ';

            carve(nx, ny, cw, ch, maze, visited, rng);
        }
    }

    // Empieza en (0,0)
    carve(0, 0, cell_w, cell_h, &mut maze, &mut visited, &mut rng);

    // Coloca 'p' y 'g'
    maze[1][1]             = 'p';
    maze[gh - 2][gw - 2]   = 'g';

    maze
}
