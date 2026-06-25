const std = @import("std");
const Io = std.Io;

const hello_world = @import("hello_world");

pub fn main() !void {
    std.debug.print("Hello,World!\r\n", .{});
}
