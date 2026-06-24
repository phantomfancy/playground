add_rules("mode.debug", "mode.release")

target("cleanarch_inherit")
    set_kind("binary")
    set_toolchains("clang")
    set_languages("clatest")
    add_headerfiles("*.h")
    add_files("*.c")
