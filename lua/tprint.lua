function tprint(t, indent, done)
    local Note = print
    local Tell = Tell

    local function show(val)
        if type(val) == "string" then
            return '"' .. val .. '"'
        else
            return tostring(val)
        end
    end

    done = done or {}
    indent = indent or 0
    for key, value in pairs(t) do
        Tell(string.rep(" ", indent))
        if type(value) == "table" and not done[value] then
            done[value] = true
            Note(show(key) .. ":")
            tprint(value, indent + 2, done)
        else
            Note(show(key) .. " = " .. show(value))
        end
    end
end

return tprint
