//! Folder contains implementation of each piece of hardware in the rocket. This is split into sensors (eg altimeter) and controllers (eg rocket engines, reaction wheels).
//! The struct definition for the flight controller is also contained herein, along with its instantive methods. However the majority of methods for FlightController are in  ./logic/ a sister folder to this one.

pub mod controllers;
pub mod flight_controller;
pub mod rocket;
pub mod sensors;
