add_rules("mode.debug", "mode.release")

target("cleanarch_inherit")
    set_kind("binary")
    add_headerfiles("*.h")
    add_files("*.c")
