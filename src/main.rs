extern crate cursive;

use cursive::{Cursive, Printer, XY};
use cursive::theme::{Color, ColorStyle};
use cursive::views::{Button, Dialog, LinearLayout, Panel};
use cursive::vec::Vec2;
use cursive::event::{Event, EventResult, MouseEvent};
use cursive::direction::Direction;
use std::time::SystemTime;

const GRID_SIZE: usize = 19;
const NB_CELL: usize = GRID_SIZE * GRID_SIZE;
const LEN_CELL: usize = 3;
const NB_DIR: usize = 8;
const ALL_DIR: [(i16, i16); NB_DIR] = [
    (0, 1),
    (0, -1),
    (1, 0),
    (-1, 0),
    (1, 1),
    (-1, -1),
    (1, -1),
    (-1, 1)
];
const OFFSET_LEFT_GAME: usize = 20;

const NB_MASK: usize = 5;
const LEN_MASK: usize = 6;
const MEMO_MASK_WHITE: [[(i16, i8); LEN_MASK]; NB_MASK] = [
    [(-3, CELL_EMPTY), (-2, CELL_WHITE), (-1, CELL_WHITE), (0, CELL_EMPTY), (1, CELL_EMPTY), (120, CELL_EMPTY)],
    [(-2, CELL_EMPTY), (-1, CELL_WHITE), (0, CELL_EMPTY), (1, CELL_WHITE), (2, CELL_EMPTY), (120, CELL_EMPTY)],
    [(-4, CELL_EMPTY), (-3, CELL_WHITE), (-2, CELL_WHITE), (-1, CELL_EMPTY), (0, CELL_EMPTY), (1, CELL_EMPTY)],
    [(-2, CELL_EMPTY), (-1, CELL_WHITE), (0, CELL_EMPTY), (1, CELL_EMPTY), (2, CELL_WHITE), (3, CELL_EMPTY)],
    [(-1, CELL_EMPTY), (0, CELL_EMPTY), (1, CELL_WHITE), (2, CELL_EMPTY), (3, CELL_WHITE), (4, CELL_EMPTY)],
];
const MEMO_MASK_BLACK: [[(i16, i8); LEN_MASK]; NB_MASK] = [
    [(-3, CELL_EMPTY), (-2, CELL_BLACK), (-1, CELL_BLACK), (0, CELL_EMPTY), (1, CELL_EMPTY), (120, CELL_EMPTY)],
    [(-2, CELL_EMPTY), (-1, CELL_BLACK), (0, CELL_EMPTY), (1, CELL_BLACK), (2, CELL_EMPTY), (120, CELL_EMPTY)],
    [(-4, CELL_EMPTY), (-3, CELL_BLACK), (-2, CELL_BLACK), (-1, CELL_EMPTY), (0, CELL_EMPTY), (1, CELL_EMPTY)],
    [(-2, CELL_EMPTY), (-1, CELL_BLACK), (0, CELL_EMPTY), (1, CELL_EMPTY), (2, CELL_BLACK), (3, CELL_EMPTY)],
    [(-1, CELL_EMPTY), (0, CELL_EMPTY), (1, CELL_BLACK), (2, CELL_EMPTY), (3, CELL_BLACK), (4, CELL_EMPTY)],
];

const DEPTH: i16 = 3;
const SCORE_CAP: i32 = 200;
const SCORE_ALIGN_1: i32 = 1;
const SCORE_ALIGN_2: i32 = 10;
const SCORE_ALIGN_3: i32 = 100;
const SCORE_ALIGN_4: i32 = 1000;
const SCORE_ALIGN_5: i32 = 100000;

const CELL_EMPTY: i8 = 0;
const CELL_WHITE: i8 = 1;
const CELL_BLACK: i8 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Player {
    White,
    Black,
}

#[derive(Clone, Copy)]
enum GameMode {
    Solo(Player),
    Multi,
}

struct GameView {
    go_grid: [[i8; GRID_SIZE]; GRID_SIZE],
    game_mode: GameMode,
    player_turn: Player,
    ia_time: u128,
    cursor_suggestion: Option<XY<i16>>,
    nb_cap_white: i16,
    nb_cap_black: i16,
    nb_turn: i16,
    end: Option<Option<Player>>,
}

fn player_to_i8(player: Player) -> i8 {
    match player {
        Player::Black => CELL_BLACK,
        Player::White => CELL_WHITE,
    }
}

fn player_to_str(player: Player) -> &'static str {
    match player {
        Player::Black => "black",
        Player::White => "white",
    }
}

fn next_player(player: Player) -> Player {
    match player {
        Player::Black => Player::White,
        Player::White => Player::Black,
    }
}

fn check_pos(grd: &[[i8; GRID_SIZE]; GRID_SIZE], p: XY<i16>, c: i8) -> bool {
    p.x >= 0 && (p.x as usize) < GRID_SIZE && p.y >= 0 && (p.y as usize) < GRID_SIZE && grd[p.y as usize][p.x as usize] == c
}

fn empty_pos(grd: &[[i8; GRID_SIZE]; GRID_SIZE]) -> [[bool; GRID_SIZE]; GRID_SIZE] {
    let mut todo: [[bool; GRID_SIZE]; GRID_SIZE] = [[false; GRID_SIZE]; GRID_SIZE];

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            todo[y][x] = grd[y][x] == CELL_EMPTY;
        }
    }

    todo
}

fn del_double_three(grd: &[[i8; GRID_SIZE]; GRID_SIZE], vld: &mut [[bool; GRID_SIZE]; GRID_SIZE], c: i8) {
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            if !vld[y][x] {
                continue;
            }
            vld[y][x] = check_double_three(grd, c, XY { x: x as i16, y: y as i16 });
        }
    }

    fn check_double_three(grd: &[[i8; GRID_SIZE]; GRID_SIZE], c: i8, xy: XY<i16>) -> bool {
        let XY { x, y } = xy;
        let memo = match c {
            CELL_BLACK => &MEMO_MASK_BLACK,
            _ => &MEMO_MASK_WHITE,
        };

        let mut dir_match: [bool; NB_DIR] = [false; NB_DIR];

        for i in 0..NB_DIR {
            let (dx, dy) = ALL_DIR[i];
            for j in 0..NB_MASK {
                let mut b = true;
                for k in 0..LEN_MASK {
                    let (co, cl) = memo[j][k];
                    if co >= 100 {
                        continue;
                    }
                    b = b && check_pos(grd, XY { x: x + dx * co, y: y + dy * co }, cl);
                }
                dir_match[i] = b || dir_match[i];
            }
        }

        let mut nb_double = 0;
        for i in 0..4 {
            if dir_match[i * 2] || dir_match[i * 2 + 1] {
                nb_double += 1;
            }
        }

        nb_double <= 1
    }
}

fn delcap(grd: &mut [[i8; GRID_SIZE]; GRID_SIZE], p: XY<i16>, player: Player) -> i16 {
    let mut nb_del: i16 = 0;

    for i in 0..NB_DIR {
        let (dx, dy) = ALL_DIR[i];

        let xy1: XY<i16> = XY { x: p.x + dx, y: p.y + dy };
        let xy2: XY<i16> = XY { x: p.x + dx * 2, y: p.y + dy * 2 };
        let xy3: XY<i16> = XY { x: p.x + dx * 3, y: p.y + dy * 3 };

        if !check_pos(grd, xy1, player_to_i8(next_player(player))) {
            continue;
        }
        if !check_pos(grd, xy2, player_to_i8(next_player(player))) {
            continue;
        }
        if !check_pos(grd, xy3, player_to_i8(player)) {
            continue;
        }
        grd[xy1.y as usize][xy1.x as usize] = CELL_EMPTY;
        grd[xy2.y as usize][xy2.x as usize] = CELL_EMPTY;
        nb_del += 2;
    }
    nb_del
}

fn countcap(grd: &[[i8; GRID_SIZE]; GRID_SIZE], p: XY<i16>, player: Player) -> i16 {
    let mut nb_del: i16 = 0;

    for i in 0..NB_DIR {
        let (dx, dy) = ALL_DIR[i];

        let xy1: XY<i16> = XY { x: p.x + dx, y: p.y + dy };
        let xy2: XY<i16> = XY { x: p.x + dx * 2, y: p.y + dy * 2 };
        let xy3: XY<i16> = XY { x: p.x + dx * 3, y: p.y + dy * 3 };

        if !check_pos(grd, xy1, player_to_i8(next_player(player))) {
            continue;
        }
        if !check_pos(grd, xy2, player_to_i8(next_player(player))) {
            continue;
        }
        if !check_pos(grd, xy3, player_to_i8(player)) {
            continue;
        }
        nb_del += 2;
    }
    nb_del
}

fn check_align_5p(grd: &[[i8; GRID_SIZE]; GRID_SIZE], c: i8) -> bool {
    let mut nba: i32;

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: x as i16, y: y as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            if nba >= 5 {
                return true;
            }
        }
    }

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: y as i16, y: x as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            if nba >= 5 {
                return true;
            }
        }
    }

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: (x + y) as i16, y: y as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            if nba >= 5 {
                return true;
            }
        }
    }

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: y as i16, y: (x + y) as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            if nba >= 5 {
                return true;
            }
        }
    }

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: (x as i16) - (y as i16), y: y as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            if nba >= 5 {
                return true;
            }
        }
    }

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: (GRID_SIZE as i16) - 1 - (y as i16), y: (x + y) as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            if nba >= 5 {
                return true;
            }
        }
    }

    false
}

// /!\ Slow
fn check_end_grd(
    grd: &[[i8; GRID_SIZE]; GRID_SIZE],
    nb_cap_white: i16,
    nb_cap_black: i16,
    player: Player,
) -> Option<Player> {
    if check_align_5p(grd, player_to_i8(next_player(player))) {
        return Some(next_player(player));
    }
    if !check_align_5p(grd, player_to_i8(player)) {
        return None;
    }
    let mut valid_next = empty_pos(grd);
    del_double_three(grd, &mut valid_next, player_to_i8(next_player(player)));

    let nb_cap_next_player = match next_player(player) {
        Player::White => nb_cap_white,
        Player::Black => nb_cap_black,
    };

    let mut cp_grd: [[i8; GRID_SIZE]; GRID_SIZE];
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            if !valid_next[y][x] {
                continue;
            }
            cp_grd = *grd;
            let nb_del = delcap(&mut cp_grd, XY { x: x as i16, y: y as i16 }, next_player(player));
            if nb_del == 0 {
                continue;
            }
            if nb_cap_next_player + nb_del >= 10 {
                return None;
            }
            if !check_align_5p(&cp_grd, player_to_i8(player)) {
                return None;
            }
        }
    }

    Some(player)
}

// SOLVER

fn del_dist_1(v: &[[bool; GRID_SIZE]; GRID_SIZE]) -> [[bool; GRID_SIZE]; GRID_SIZE] {
    let mut todo: [[bool; GRID_SIZE]; GRID_SIZE] = [[false; GRID_SIZE]; GRID_SIZE];

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            for j in 0..NB_DIR {
                let (dx, dy) = ALL_DIR[j];
                let xx = (x as i16) + dx;
                let yy = (y as i16) + dy;
                todo[y][x] = todo[y][x] || !v[y][x] ||
                    (xx >= 0 && (xx as usize) < GRID_SIZE && yy >= 0 && (yy as usize) < GRID_SIZE &&
                        !v[yy as usize][xx as usize]
                    );
            }
        }
    }

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            todo[y][x] = todo[y][x] && v[y][x];
        }
    }

    todo
}

fn valid_to_pos(v: &[[bool; GRID_SIZE]; GRID_SIZE]) -> Vec<XY<i16>> {
    let mut todo: Vec<XY<i16>> = Vec::new();

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            if v[y][x] {
                todo.push(XY { x: x as i16, y: y as i16 });
            }
        }
    }

    todo
}

fn nba_to_score(nba: i32) -> i32 {
    match nba {
        0 => 0,
        1 => SCORE_ALIGN_1,
        2 => SCORE_ALIGN_2,
        3 => SCORE_ALIGN_3,
        4 => SCORE_ALIGN_4,
        _ => SCORE_ALIGN_5,
    }
}

fn scoring_align(grd: &[[i8; GRID_SIZE]; GRID_SIZE], player: Player) -> i32 {
    let mut score: i32 = 0;
    let c = player_to_i8(player);
    let mut nba: i32;

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: x as i16, y: y as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            score += nba_to_score(nba);
        }
    }

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: y as i16, y: x as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            score += nba_to_score(nba);
        }
    }

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: (x + y) as i16, y: y as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            score += nba_to_score(nba);
        }
    }

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: y as i16, y: (x + y) as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            score += nba_to_score(nba);
        }
    }

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: (x as i16) - (y as i16), y: y as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            score += nba_to_score(nba);
        }
    }

    for x in 0..GRID_SIZE {
        nba = 0;
        for y in 0..GRID_SIZE {
            if check_pos(grd, XY { x: (GRID_SIZE as i16) - 1 - (y as i16), y: (x + y) as i16 }, c) {
                nba += 1;
            } else {
                nba = 0;
            }
            score += nba_to_score(nba);
        }
    }

    score
}

fn scoring_ordoring(grd: &[[i8; GRID_SIZE]; GRID_SIZE], player: Player, p: XY<i16>) -> i32 {
    let XY { x, y } = p;
    let mut score: i32 = 0;
    let mut nba: i16 = 0;

    for i in 0..NB_DIR {
        let (dx, dy) = ALL_DIR[i];
        nba = 0;

        loop {
            if !check_pos(grd, XY { x: x + dx * nba, y: y + dy * nba }, CELL_BLACK) {
                break;
            }
            nba += 1;
            score += nba_to_score(nba as i32);
        }
        nba = 0;
        loop {
            if !check_pos(grd, XY { x: x + dx * nba, y: y + dy * nba }, CELL_WHITE) {
                break;
            }
            nba += 1;
            score += nba_to_score(nba as i32);
        }
    }

    score
}

fn scoring_end(
    grd: &[[i8; GRID_SIZE]; GRID_SIZE],
    nb_cap_white: i16,
    nb_cap_black: i16,
    player: Player,
) -> i32 {
    let mut score: i32 = match player {
        Player::White => ((nb_cap_white - nb_cap_black) as i32) * SCORE_CAP,
        Player::Black => ((nb_cap_black - nb_cap_white) as i32) * SCORE_CAP,
    };
    score += scoring_align(grd, player);
    score -= scoring_align(grd, next_player(player));

    score
}

fn nega_max(
    grd: &[[i8; GRID_SIZE]; GRID_SIZE],
    nb_cap_white: i16,
    nb_cap_black: i16,
    depth: i16,
    alpha: i32,
    beta: i32,
    player: Player,
) -> (XY<i16>, i32) {
    let mut alpha_mut = alpha;
    let mut to_find: (XY<i16>, i32) = (XY { x: 0, y: 0 }, std::i32::MIN + 100);
    let mut cp: [[i8; GRID_SIZE]; GRID_SIZE];

    if nb_cap_black >= 10 {
        if player == Player::Black {
            return (XY { x: 0, y: 0 }, std::i32::MAX / 2);
        } else {
            return (XY { x: 0, y: 0 }, std::i32::MIN / 2);
        }
    }
    if nb_cap_white >= 10 {
        if player == Player::White {
            return (XY { x: 0, y: 0 }, std::i32::MAX / 2);
        } else {
            return (XY { x: 0, y: 0 }, std::i32::MIN / 2);
        }
    }
    if depth <= 0 {
        return (XY { x: 0, y: 0 }, scoring_end(grd, nb_cap_white, nb_cap_black, player));
    }

    // need move
    if let Some(p) = check_end_grd(grd, nb_cap_white, nb_cap_black, player) {
        if p == player {
            return (XY { x: 0, y: 0 }, std::i32::MAX / 2);
        } else {
            return (XY { x: 0, y: 0 }, std::i32::MIN / 2);
        }
    }


    let mut valid = empty_pos(&grd);
    valid = del_dist_1(&valid);
    del_double_three(&grd, &mut valid, player_to_i8(player));
    let lpos = valid_to_pos(&valid);


    let mut lpos_score: Vec<(XY<i16>, i32)> = Vec::new();
    for XY { x, y } in lpos.iter() {
        lpos_score.push((XY { x: *x, y: *y }, scoring_ordoring(grd, player, XY { x: *x, y: *y })))
    }
    lpos_score.sort_by_key(|k| k.1);

    for (XY { x, y }, _) in lpos_score.iter().rev() {
        cp = *grd;
        cp[*y as usize][*x as usize] = player_to_i8(player);
        let cap = delcap(&mut cp, XY { x: *x, y: *y }, player);

        let (_, s) = nega_max(
            &cp,
            if player == Player::White { nb_cap_white + cap } else { nb_cap_white },
            if player == Player::Black { nb_cap_black + cap } else { nb_cap_black },
            depth - 1,
            -beta,
            -alpha_mut,
            next_player(player),
        );
        let ss = -s;
        if ss > to_find.1 {
            to_find = (XY { x: *x, y: *y }, ss);
        }
        alpha_mut = alpha_mut.max(ss);
        if alpha_mut >= beta {
            break;
        }
    }

    to_find
}

// SOLVER

impl GameView {
    pub fn new(game_mode: GameMode) -> Self {
        let mut gv = GameView {
            go_grid: [[CELL_EMPTY; GRID_SIZE]; GRID_SIZE],
            game_mode,
            player_turn: Player::Black,
            ia_time: 0,
            cursor_suggestion: None,
            nb_cap_white: 0,
            nb_cap_black: 0,
            nb_turn: 2,
            end: None,
        };

        if let GameMode::Solo(Player::White) = game_mode {
            gv.go_grid[GRID_SIZE / 2][GRID_SIZE / 2] = CELL_BLACK;
            gv.nb_turn += 1;
            gv.player_turn = Player::White;
        }
        gv
    }

    pub fn handle_player_play(&mut self, p: XY<i16>) {
        if self.end != None {
            return;
        }

        let mut valid = empty_pos(&self.go_grid);
        del_double_three(&self.go_grid, &mut valid, player_to_i8(self.player_turn));
        if !valid[p.y as usize][p.x as usize] {
            return;
        }

        self.go_grid[p.y as usize][p.x as usize] = player_to_i8(self.player_turn);
        let cap = delcap(&mut self.go_grid, p, self.player_turn);

        if self.player_turn == Player::Black {
            self.nb_cap_black += cap;
        } else {
            self.nb_cap_white += cap;
        }

        if self.nb_cap_black >= 10 {
            self.end = Some(Some(Player::Black));
            return;
        }
        if self.nb_cap_white >= 10 {
            self.end = Some(Some(Player::White));
            return;
        }
        if let Some(p) = check_end_grd(&self.go_grid, self.nb_cap_white, self.nb_cap_black, self.player_turn) {
            self.end = Some(Some(p));
            return;
        }

        self.player_turn = next_player(self.player_turn);
        self.nb_turn += 1;

        if let GameMode::Multi = self.game_mode {
            return;
        }

        let now = SystemTime::now();

        let (xy_ia, _) = nega_max(
            &self.go_grid,
            self.nb_cap_white,
            self.nb_cap_black,
            DEPTH,
            std::i32::MIN / 2,
            std::i32::MAX / 2,
            self.player_turn,
        );

        match now.elapsed() {
            Ok(d) => self.ia_time = d.as_millis(),
            Err(_e) => (),
        }

        self.go_grid[xy_ia.y as usize][xy_ia.x as usize] = player_to_i8(self.player_turn);
        let cap = delcap(&mut self.go_grid, xy_ia, self.player_turn);
        if self.player_turn == Player::Black {
            self.nb_cap_black += cap;
        } else {
            self.nb_cap_white += cap;
        }

        if self.nb_cap_black >= 10 {
            self.end = Some(Some(Player::Black));
            return;
        }
        if self.nb_cap_white >= 10 {
            self.end = Some(Some(Player::White));
            return;
        }
        if let Some(p) = check_end_grd(&self.go_grid, self.nb_cap_white, self.nb_cap_black, self.player_turn) {
            self.end = Some(Some(p));
            return;
        }

        self.player_turn = next_player(self.player_turn);
        self.nb_turn += 1;
    }
}

impl cursive::view::View for GameView {
    fn draw(&self, printer: &Printer) {
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let cell = self.go_grid[y][x];

                let text = match cell {
                    CELL_EMPTY => " o ",
                    CELL_WHITE => "( )",
                    _ => "( )",
                };

                let color_back = match cell {
                    CELL_EMPTY => Color::Rgb(200, 200, 200),
                    CELL_WHITE => Color::RgbLowRes(5, 5, 5),
                    _ => Color::RgbLowRes(0, 0, 0),
                };

                let color_font = match cell {
                    CELL_EMPTY => Color::RgbLowRes(0, 0, 0),
                    CELL_WHITE => Color::RgbLowRes(3, 3, 3),
                    _ => Color::RgbLowRes(2, 2, 2),
                };

                printer.with_color(
                    ColorStyle::new(color_font, color_back),
                    |printer| printer.print((x * LEN_CELL + OFFSET_LEFT_GAME, y), text),
                );
            }
        }

        fn print_tmp(printer: &Printer, p: (usize, usize), text: &str) {
            let color_back = Color::Rgb(200, 200, 200);
            let color_font = Color::RgbLowRes(0, 0, 0);
            printer.with_color(
                ColorStyle::new(color_font, color_back),
                |printer| printer.print(p, text),
            );
        }

        print_tmp(printer, (0, 1), &format!("Turn N°: {}", (self.nb_turn / 2))[..]);
        print_tmp(printer, (0, 2), &format!("Turn: Player {}", player_to_str(self.player_turn))[..]);
        print_tmp(printer, (0, 3), &format!("Nb cap Black: {}", self.nb_cap_black)[..]);
        print_tmp(printer, (0, 4), &format!("Nb cap White: {}", self.nb_cap_white)[..]);
        print_tmp(printer, (0, 6), &format!("Time IA: {} ms", self.ia_time)[..]);

        if let Some(end) = self.end {
            match end {
                None => print_tmp(printer, (0, 8), "Draw"),
                Some(p) => print_tmp(printer, (0, 8), &format!("Player {} win!", player_to_str(p))[..]),
            }
        }
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        Vec2::new(GRID_SIZE * LEN_CELL + OFFSET_LEFT_GAME, GRID_SIZE)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Mouse {
                offset,
                position,
                event: MouseEvent::Release(_btn),
            } => {
                let pos = position
                    .checked_sub(offset)
                    .map(|pos| pos.map_x(|x| {
                        if x > OFFSET_LEFT_GAME {
                            (x - OFFSET_LEFT_GAME) / LEN_CELL
                        } else {
                            1024
                        }
                    }));

                if let Some(p) = pos {
                    if p.y < GRID_SIZE && p.x < GRID_SIZE {
                        self.handle_player_play(XY { x: p.x as i16, y: p.y as i16 });
                    }
                }
            }
            _ => (),
        }

        EventResult::Ignored
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }
}

fn display_game(siv: &mut Cursive, game_mode: GameMode) {
    siv.add_layer(
        Dialog::new()
            .title("Gomoku")
            .padding((6, 6, 2, 2))
            .content(
                LinearLayout::horizontal()
                    .child(Panel::new(GameView::new(game_mode))),
            )
            .button("Quit game", |s| {
                s.pop_layer();
            }),
    );
}

fn display_turn_choice(siv: &mut Cursive) {
    siv.add_layer(
        Dialog::new()
            .title("Player Turn")
            .padding((2, 2, 1, 1))
            .content(
                LinearLayout::vertical()
                    .child(Button::new_raw(" First (black) ", |s| display_game(s, GameMode::Solo(Player::Black))))
                    .child(Button::new_raw(" Second (white) ", |s| display_game(s, GameMode::Solo(Player::White))))
                    .child(Button::new_raw("     Back      ", |s| { s.pop_layer(); })),
            ),
    );
}

fn display_home(siv: &mut Cursive) {
    siv.add_layer(
        Dialog::new()
            .title("Gomoku")
            .padding((2, 2, 1, 1))
            .content(
                LinearLayout::vertical()
                    .child(Button::new_raw(" Multiplayer ", |s| display_game(s, GameMode::Multi)))
                    .child(Button::new_raw("    Solo    ", display_turn_choice))
                    .child(Button::new_raw("    Exit     ", |s| s.quit())),
            ),
    );
}

fn main() {
    let mut siv = Cursive::default();
    display_home(&mut siv);
    siv.run();
}
