use dutchy_egui::{ScribbleBrush, ScribbleDrawing, ScribbleStroke};
use egui::pos2;
pub fn drawing() -> ScribbleDrawing {
    let mut drawing = ScribbleDrawing::default();
    for points in [
        vec![
            pos2(105.0, 150.0),
            pos2(116.0, 70.0),
            pos2(140.0, 35.0),
            pos2(180.0, 42.0),
            pos2(207.0, 68.0),
            pos2(210.0, 105.0),
            pos2(185.0, 135.0),
            pos2(145.0, 148.0),
            pos2(112.0, 148.0),
        ],
        vec![
            pos2(250.0, 58.0),
            pos2(292.0, 39.0),
            pos2(325.0, 51.0),
            pos2(318.0, 72.0),
            pos2(265.0, 91.0),
            pos2(250.0, 112.0),
            pos2(265.0, 137.0),
            pos2(310.0, 143.0),
            pos2(344.0, 123.0),
        ],
        vec![
            pos2(85.0, 171.0),
            pos2(175.0, 165.0),
            pos2(275.0, 168.0),
            pos2(375.0, 158.0),
            pos2(470.0, 160.0),
        ],
    ] {
        drawing
            .push(ScribbleStroke::new(points, ScribbleBrush::new()).unwrap())
            .unwrap();
    }
    drawing
}
