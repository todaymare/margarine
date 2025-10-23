use std::{collections::HashMap, ffi::CString, str::FromStr};

use raylib::{color, ffi::KeyboardKey};
use runtime::{Reg, Status, VM};

pub fn raylib(hosts: &mut HashMap<String, unsafe extern "C" fn(&mut VM, &mut Reg, &mut Status)>) {
    unsafe extern "C" fn init_window(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let width = vm.stack.reg(0).as_int();
        let height = vm.stack.reg(1).as_int();
        let title = vm.stack.reg(2).as_obj();
        let title = vm.objs[title].as_str();

        raylib::ffi::InitWindow(
            width as _,
            height as _,
            CString::new(title).unwrap().as_ptr(),
        );
    }

    unsafe extern "C" fn close_window(_: &mut VM, _: &mut Reg, _: &mut Status) {
        raylib::ffi::CloseWindow();
    }

    unsafe extern "C" fn window_should_close(_: &mut VM, ret: &mut Reg, _: &mut Status) {
        let value = raylib::ffi::WindowShouldClose();
        *ret = Reg::new_bool(value);
    }

    unsafe extern "C" fn begin_drawing(_: &mut VM, _: &mut Reg, _: &mut Status) {
        raylib::ffi::BeginDrawing();
    }

    unsafe extern "C" fn end_drawing(_: &mut VM, _: &mut Reg, _: &mut Status) {
        raylib::ffi::EndDrawing();
    }

    unsafe extern "C" fn clear_background(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let colour = vm.stack.reg(0).as_obj();
        let colour = vm.objs[colour].as_fields();
        let r = (colour[0].as_float() * 255.999) as u8;
        let g = (colour[1].as_float() * 255.999) as u8;
        let b = (colour[2].as_float() * 255.999) as u8;
        let a = (colour[3].as_float() * 255.999) as u8;
        raylib::ffi::ClearBackground(raylib::ffi::Color { r, g, b, a });
    }

    unsafe extern "C" fn set_target_fps(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let fps = vm.stack.reg(0).as_int();
        raylib::ffi::SetTargetFPS(fps as _);
    }

    unsafe extern "C" fn draw_text(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let text = vm.stack.reg(0).as_obj();
        let text = vm.objs[text].as_str();
        let pos_x = vm.stack.reg(1).as_int();
        let pos_y = vm.stack.reg(2).as_int();
        let font_size = vm.stack.reg(3).as_int();
        let colour = vm.stack.reg(4).as_obj();
        let colour = vm.objs[colour].as_fields();
        let r = (colour[0].as_float() * 255.999) as u8;
        let g = (colour[1].as_float() * 255.999) as u8;
        let b = (colour[2].as_float() * 255.999) as u8;
        let a = (colour[3].as_float() * 255.999) as u8;

        raylib::ffi::DrawText(
            CString::new(text).unwrap().as_ptr(),
            pos_x as _,
            pos_y as _,
            font_size as _,
            raylib::ffi::Color { r, g, b, a },
        );
    }

    unsafe extern "C" fn draw_rectangle(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let x = vm.stack.reg(0).as_int();
        let y = vm.stack.reg(1).as_int();
        let w = vm.stack.reg(2).as_int();
        let h = vm.stack.reg(3).as_int();
        let colour = vm.stack.reg(4).as_obj();
        let colour = vm.objs[colour].as_fields();
        let r = (colour[0].as_float() * 255.999) as u8;
        let g = (colour[1].as_float() * 255.999) as u8;
        let b = (colour[2].as_float() * 255.999) as u8;
        let a = (colour[3].as_float() * 255.999) as u8;

        raylib::ffi::DrawRectangle(
            x as _,
            y as _,
            w as _,
            h as _,
            raylib::ffi::Color { r, g, b, a },
        );
    }

    unsafe extern "C" fn draw_fps(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let x = vm.stack.reg(0).as_int();
        let y = vm.stack.reg(1).as_int();
        raylib::ffi::DrawFPS(x as _, y as _);
    }

    unsafe extern "C" fn is_key_pressed(vm: &mut VM, ret: &mut Reg, _: &mut Status) {
        let key = vm.stack.reg(0).as_int();
        *ret = Reg::new_bool(raylib::ffi::IsKeyPressed(key as _));
    }

    unsafe extern "C" fn frame_time(_: &mut VM, ret: &mut Reg, _: &mut Status) {
        *ret = Reg::new_float(raylib::ffi::GetFrameTime() as _);
    }

    hosts.insert("RaylibInitWindow".to_string(), init_window);
    hosts.insert("RaylibCloseWindow".to_string(), close_window);
    hosts.insert("RaylibWindowShouldClose".to_string(), window_should_close);
    hosts.insert("RaylibBeginDrawing".to_string(), begin_drawing);
    hosts.insert("RaylibEndDrawing".to_string(), end_drawing);
    hosts.insert("RaylibClearBackground".to_string(), clear_background);
    hosts.insert("RaylibSetTargetFPS".to_string(), set_target_fps);
    hosts.insert("RaylibDrawText".to_string(), draw_text);
    hosts.insert("RaylibDrawRectangle".to_string(), draw_rectangle);
    hosts.insert("RaylibDrawFPS".to_string(), draw_fps);
    hosts.insert("RaylibIsKeyPressed".to_string(), is_key_pressed);
    hosts.insert("RaylibFrameTime".to_string(), frame_time);
}
