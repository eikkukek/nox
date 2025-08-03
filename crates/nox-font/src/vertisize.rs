use core::{
    cmp::Ordering,
    fmt::Display,
};

use nox_mem::{Vector, vec_types::GlobalVec};

use super::*;

#[derive(Clone, Copy, Debug)]
struct Line {
    p0: [f32; 2],
    p1: [f32; 2],
}

impl Line {

    pub fn new(p0: [f32; 2], p1: [f32; 2]) -> Self {
        if p0[1] > p1[1] {
            Self {
                p0: p1,
                p1: p0,
            }
        }
        else {
            Self {
                p0,
                p1,
            }
        }
    }
}

impl PartialEq for Line {

    fn eq(&self, other: &Self) -> bool {
        self.p0[1] == other.p0[1]
    }
}

impl PartialOrd for Line {
    
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.p0[1], &other.p0[1])
    }
}

#[derive(Debug)]
struct Outline {
    lines: GlobalVec<Line>,
}

impl Outline {
    
    fn new() -> Self {
        Self {
            lines: GlobalVec::new(),
        }
    }

    fn insert_line(&mut self, line: Line) {
        if let Some((i, _)) = self.lines.iter().enumerate().find(|f| f.1 > &line) {
            self.lines.insert(line, i).unwrap();
        }
        else {
            self.lines.push(line).unwrap();
        }
    }
}

struct OutlineBuilder {
    outlines: GlobalVec<Outline>,
    current_outline: Option<Outline>,
    pos: [f32; 2],
    curve_steps: u32,
    curve_step: f32,
}

impl OutlineBuilder {

    fn new(curve_steps: u32) -> Self {
        Self {
            outlines: GlobalVec::new(),
            current_outline: Some(Outline::new()),
            pos: [0.0; 2],
            curve_steps,
            curve_step: 1.0 / curve_steps as f32,
        }
    }

    fn insert_line(&mut self, line: Line) {
        unsafe {
            self.current_outline
                .as_mut()
                .unwrap_unchecked()
                .insert_line(line);
        }
    }
}

impl ttf_parser::OutlineBuilder for OutlineBuilder {

    fn move_to(&mut self, x: f32, y: f32) {
        self.pos = [x, y];
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let end = [x, y];
        let line = Line::new(self.pos, end);
        self.insert_line(line);
        self.pos = end;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p0 = self.pos;
        let p1 = [x, y];
        let p2 = [x1, y1];
        let step = self.curve_step;
        let mut pos = p0;
        let mut t0 = 0.0;
        for _ in 0..self.curve_steps - 1 {
            let t0p2 = t0 * t0;
            let t1 = 1.0 - t0;
            let t1p2 = t1 * t1;
            let tmp = [
                p1[0] + t1p2 * (p0[0] - p1[0]) + t0p2 * (p2[0] - p1[0]),
                p1[1] + t1p2 * (p0[1] - p1[1]) + t0p2 * (p2[1] - p1[1]),
            ];
            self.insert_line(Line::new(pos, tmp));
            pos = tmp;
            t0 += step;
        }
        self.insert_line(Line::new(pos, p1));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let p0 = self.pos;
        let p1 = [x, y];
        let p2 = [x1, y1];
        let p3 = [x2, y2];
        let step = self.curve_step;
        let mut pos = p0;
        let mut t0 = 0.0;
        for _ in 0..self.curve_steps - 1 {
            let t0p2 = t0 * t0;
            let t1 = 1.0 - t0;
            let t1p2 = t1 * t1;
            let num0 = t1 * t1 * t1;
            let num1 = 3.0 * t1p2 * t0;
            let num2 = 3.0 * t1 * t0p2;
            let num3 = t0p2 * t0;
            let tmp = [
                num0 * p0[0] + num1 * p1[0] + num2 * p2[0] + num3 * p3[0],
                num0 * p0[1] + num1 * p1[1] + num2 * p2[1] + num3 * p3[1],
            ];
            self.insert_line(Line::new(pos, tmp));
            pos = tmp;
            t0 += step;
        }
        self.insert_line(Line::new(pos, p1));
    }

    fn close(&mut self) {
        self.outlines
            .push(self.current_outline.take().unwrap())
            .unwrap();
        self.current_outline = Some(Outline::new());
    }
}

pub fn vertisize(
    glyph: char,
    curve_steps: u32,
    face: &Face
) -> Option<GlobalVec<[f32; 2]>>
{
    let id = face.glyph_index(glyph)?;
    let mut builder = OutlineBuilder::new(curve_steps);
    face.outline_glyph(id, &mut builder)?;
    println!("{:?}", builder.outlines[1]);
    Some(GlobalVec::new())
}
