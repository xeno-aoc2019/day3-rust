use std::fmt;
use std::fs::File;
use std::path::Path;
use std::io::{Lines, BufRead, BufReader, Result};
use std::cmp::{min, max};

#[derive(Copy, Debug, Clone)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Copy, Debug, Clone)]
struct PointWithCost {
    point: Point,
    cost: i32,
}

#[derive(Copy, Debug, Clone)]
struct PathSegment {
    direction: char,
    distance: i32,
}

#[derive(Copy, Debug, Clone)]
struct Segment {
    end1: Point,
    end2: Point,
    steps: i32,
    mirrored: bool,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ms = if self.mirrored { "<>" } else { "" };
        write!(f, "({}-{}#{}{})", self.end1, self.end2, self.steps, ms)
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {})", self.direction, self.distance)
    }
}

fn toPathSegment(c: &str) -> PathSegment {
    let direction = c.chars().next().unwrap();
    let distance: i32 = c.trim()[1..].parse().unwrap();
    return PathSegment { direction, distance };
}

fn path_to_segments(path: Vec<PathSegment>) -> Vec<Segment> {
    let mut segments: Vec<Segment> = vec!();
    let mut curr = Point { x: 0, y: 0 };
    let mut steps = 0;
    for step in path {
        let next = match step.direction {
            'U' => Point { x: curr.x, y: curr.y + step.distance },
            'D' => Point { x: curr.x, y: curr.y - step.distance },
            'L' => Point { x: curr.x - step.distance, y: curr.y },
            'R' => Point { x: curr.x + step.distance, y: curr.y },
            _ => panic!("unknown direction")
        };
        let segment = Segment { end1: curr, end2: next, steps, mirrored: false };
        steps += step.distance;
        println!("{} => {} ", step, segment);
        segments.push(segment);
        curr = next;
    }
    return segments;
}

fn normalize(segments: Vec<Segment>) -> Vec<Segment> {
    let mut normalized: Vec<Segment> = vec!();
    for segment in segments {
        if segment.end1.x > segment.end2.x || segment.end1.y > segment.end2.y {
            normalized.push(Segment { end1: segment.end2, end2: segment.end1, steps: segment.steps, mirrored: true });
        } else {
            normalized.push(segment);
        }
    }
    normalized
}

fn split_on_direction(segments: Vec<Segment>) -> (Vec<Segment>, Vec<Segment>) {
    let mut horizontals: Vec<Segment> = vec!();
    let mut verticals: Vec<Segment> = vec!();
    let mut intersects: Vec<Point> = vec!();
    for segment in segments {
        if segment.end1.x == segment.end2.x {
            verticals.push(segment);
        } else {
            horizontals.push(segment);
        }
    }
    (horizontals, verticals)
}

fn cost_for_segment(p: Point, s: &Segment) -> i32 {
    if s.end1.x != s.end2.x {
        // horizontal
        if s.mirrored {
            return s.steps + s.end2.x - p.x;
        } else {
            return s.steps + p.x - s.end1.x;
        }
    } else {
        // vertical
        if s.mirrored {
            return s.steps + s.end2.y - p.y;
        } else {
            return s.steps + p.y - s.end1.y;
        }
    }
}

fn cost(p: Point, segment1: &Segment, segment2: &Segment) -> i32 {
    return cost_for_segment(p, segment1) + cost_for_segment(p, segment2);
}

fn intersects_horizontal(segment: Segment, horizontals: &Vec<Segment>, verticals: &Vec<Segment>) -> Vec<PointWithCost> {
    let mut intersects: Vec<PointWithCost> = vec!();
    for other in horizontals {
        if segment.end1.y == other.end1.y {
            let leftmost = max(segment.end1.x, other.end1.x);
            let rightmost = min(segment.end2.x, other.end2.x);
            let point1 = Point { x: leftmost, y: segment.end1.y };
            let point2 = Point { x: rightmost, y: segment.end1.y };
            let cost1 = cost(point1, other, &segment);
            let cost2 = cost(point2, other, &segment);
            intersects.push(PointWithCost { point: point1, cost: cost1 });
            intersects.push(PointWithCost { point: point2, cost: cost2 });
        }
    }
    for other in verticals {
        if between(other.end1.x, segment.end1.x, segment.end2.x) {
            if between(segment.end1.y, other.end1.y, other.end2.y) {
                let point = Point { x: other.end1.x, y: segment.end1.y };
                let cost1 = cost(point, other, &segment);
                intersects.push(PointWithCost { point, cost: cost1 });
            }
        }
    }
    intersects
}

fn intersects_vertical(segment: Segment, horizontals: &Vec<Segment>, verticals: &Vec<Segment>) -> Vec<PointWithCost> {
    let mut intersects: Vec<PointWithCost> = vec!();
    for other in verticals {
        if segment.end1.x == other.end1.x {
            let bottom = max(segment.end1.y, other.end1.y);
            let top = min(segment.end2.y, other.end2.y);
            let point1 = Point { x: segment.end1.x, y: bottom };
            let point2 = Point { x: segment.end1.x, y: top };
            println!("intersect: {},{} -> {} {} ", segment, other, point1, point2);
            let cost1 = cost(point1, &segment, other);
            let cost2 = cost(point2, &segment, other);
            intersects.push(PointWithCost { point: point1, cost: cost1 });
            intersects.push(PointWithCost { point: point2, cost: cost2 });
        }
    }
    for other in horizontals {
        if between(other.end1.y, segment.end1.y, segment.end2.y) {
            if between(segment.end1.x, other.end1.x, other.end2.x) {
                let point = Point { x: segment.end1.x, y: other.end1.y };
                let cost1 = cost(point, &segment, other);
                intersects.push(PointWithCost { point, cost: cost1 });
            }
        }
    }
    intersects
}

fn intersects(segment: Segment, horizontals: &Vec<Segment>, verticals: &Vec<Segment>) -> Vec<PointWithCost> {
    if segment.end1.x == segment.end2.x {
        println!("Vertical for : {}", segment);
        return intersects_vertical(segment, &horizontals, &verticals);
    } else {
        println!("Horizontal for : {}", segment);
        return intersects_horizontal(segment, &horizontals, &verticals);
    }
}

fn between(i: i32, low: i32, high: i32) -> bool {
    if i < low { return false; }
    if i > high { return false; }
    return true;
}

fn distance(p: Point) -> i32 {
    p.x.abs() + p.y.abs()
}

fn closest_intersect(path1: Vec<Segment>, path2: Vec<Segment>) -> (i32, PointWithCost, PointWithCost) {
    let (horizontals, verticals) = split_on_direction(path2);
    let mut closest_intersect = PointWithCost { point: Point { x: 10000000, y: 10000000 }, cost: 10000000 };
    let mut closest_by_path = PointWithCost { point: Point { x: 10000000, y: 10000000 }, cost: 10000000 };
    let mut closest_distance = 100000000;
    for segment in path1 {
        let is = intersects(segment, &horizontals, &verticals);
        for i in is {
            let dist = distance(i.point);
            if dist < closest_distance && dist != 0 {
                closest_distance = dist;
                closest_intersect = i;
            }
            if i.cost < closest_by_path.cost && i.cost > 0 {
                closest_by_path = i;
            }
        }
    }
    (closest_distance, closest_intersect, closest_by_path)
}

fn main() {
    let mut path_strings: Vec<String> = vec!();
    if let Ok(lines) = read_lines("input.txt") {
        for line in lines {
            if let Ok(path_string) = line {
                path_strings.push(path_string);
            }
        }
    }
//    let path_strings_0: Vec<PathSegment> = path_strings[0].split(',').map(toPathSegment).collect();
//    let path_strings_1: Vec<PathSegment> = path_strings[1].split(',').map(toPathSegment).collect();
    let path_strings_0: Vec<PathSegment> = "R75,D30,R83,U83,L12,D49,R71,U7,L72".split(',').map(toPathSegment).collect();
    let path_strings_1: Vec<PathSegment> = "U62,R66,U55,R34,D71,R55,D58,R83".split(',').map(toPathSegment).collect();

    let segments_0 = normalize(path_to_segments(path_strings_0));
    let segments_1 = normalize(path_to_segments(path_strings_1));
    println!("{}", segments_0.len());
    println!("{}", segments_1.len());
    let (dist, inter, inter2) = closest_intersect(segments_0, segments_1);
    println!("TASK 1: dist: {} for {}", dist, inter.point);
    println!("TASK 2: dist: {} for {}", inter2.cost, inter2.point);
}

fn read_lines<P>(filename: P) -> Result<Lines<BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}