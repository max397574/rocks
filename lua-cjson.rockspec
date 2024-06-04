package = "lua-cjson"
version = "2.1.0.11-1"

source = {
    url = "git+https://github.com/openresty/lua-cjson",
    tag = "2.1.0.11",
}

description = {
    summary = "A fast JSON encoding/parsing module",
    detailed = [[
        The Lua CJSON module provides JSON support for Lua. It features:
        - Fast, standards compliant encoding/parsing routines
        - Full support for JSON with UTF-8, including decoding surrogate pairs
        - Optional run-time support for common exceptions to the JSON specification
          (infinity, NaN,..)
        - No dependencies on other libraries
    ]],
    homepage = "http://www.kyne.com.au/~mark/software/lua-cjson.php",
    license = "MIT"
}

dependencies = {
    "lua >= 5.1"
}

build = {
    type = "builtin",
    modules = {
        cjson = {"lua_cjson.c", "strbuf.c", "fpconv.c"},
        ["cjson.safe"] ={"lua_cjson.c", "strbuf.c", "fpconv.c" },
    },
    -- Override default build options (per platform)
    -- platforms = {
    --     win32 = { modules = { cjson = { defines = {
    --         "DISABLE_INVALID_NUMBERS", "USE_INTERNAL_ISINF"
    --     } } } }
    -- },
    copy_directories = { "tests" }
}