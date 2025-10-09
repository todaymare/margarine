use std::{collections::HashMap, ffi::CString, str::FromStr};

use raylib::color;
use runtime::{Reg, VM};

pub fn raylib(hosts: &mut HashMap<String, fn(&mut VM) -> Reg>) {
    hosts.insert("RaylibInitWindow".to_string(),
    |vm| { unsafe {
        let width = vm.stack.reg(0).as_int();
        let height = vm.stack.reg(1).as_int();
        let title = vm.stack.reg(2).as_obj();
        let title = vm.objs[title as usize].as_str();

        dbg!(title);
        raylib::ffi::InitWindow(
            width as _,
            height as _,
            CString::from_str(title).unwrap().as_ptr(),
        );

        Reg::new_unit()
    } });

    hosts.insert("RaylibCloseWindow".to_string(),
    |_| { unsafe {
        raylib::ffi::CloseWindow();
        Reg::new_unit()
    } });

    hosts.insert("RaylibWindowShouldClose".to_string(),
    |_| { unsafe {
        let value = raylib::ffi::WindowShouldClose();
        Reg::new_bool(value)
    } });

    hosts.insert("RaylibBeginDrawing".to_string(),
    |_| { unsafe {
        raylib::ffi::BeginDrawing();
        Reg::new_unit()
    } });

    hosts.insert("RaylibEndDrawing".to_string(),
    |_| { unsafe {
        raylib::ffi::EndDrawing();
        Reg::new_unit()
    } });

    hosts.insert("RaylibClearBackground".to_string(),
    |vm| { unsafe {
        let colour = vm.stack.reg(0).as_obj();
        let colour = vm.objs[colour as usize].as_fields();
        let r = (colour[0].as_float() * 255.999) as u8;
        let g = (colour[1].as_float() * 255.999) as u8;
        let b = (colour[2].as_float() * 255.999) as u8;
        let a = (colour[3].as_float() * 255.999) as u8;
        raylib::ffi::ClearBackground(raylib::ffi::Color { r, g, b, a });
        Reg::new_unit()
    } });

    hosts.insert("RaylibSetTargetFPS".to_string(),
    |vm| { unsafe {
        let fps = vm.stack.reg(0).as_int();
        raylib::ffi::SetTargetFPS(fps as _);
        Reg::new_unit()
    } });

    hosts.insert("RaylibDrawText".to_string(),
    |vm| { unsafe {
        let text = vm.stack.reg(0).as_obj();
        let text = vm.objs[text as usize].as_str();

        let pos_x = vm.stack.reg(1).as_int();
        let pos_y = vm.stack.reg(2).as_int();
        let font_size = vm.stack.reg(3).as_int();

        let colour = vm.stack.reg(4).as_obj();
        let colour = vm.objs[colour as usize].as_fields();
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

        Reg::new_unit()
    } });

    hosts.insert("RaylibDrawRectangle".to_string(),
    |vm| { unsafe {
        let x = vm.stack.reg(0).as_int();
        let y = vm.stack.reg(1).as_int();
        let w = vm.stack.reg(2).as_int();
        let h = vm.stack.reg(3).as_int();

        let colour = vm.stack.reg(4).as_obj();
        let colour = vm.objs[colour as usize].as_fields();
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
        Reg::new_unit()
    } });

    hosts.insert("RaylibDrawFPS".to_string(),
    |vm| { unsafe {
        let x = vm.stack.reg(0).as_int();
        let y = vm.stack.reg(1).as_int();

        raylib::ffi::DrawFPS(x as _, y as _);
        Reg::new_unit()
    } });


    hosts.insert("RaylibIsKeyPressed".to_string(),
    |vm| { unsafe {
        let x = vm.stack.reg(0).as_int();
        Reg::new_bool(raylib::ffi::IsKeyPressed(x as _))
    } });


}
