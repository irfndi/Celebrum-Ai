-- Simulate different user types
local user_types = {"free", "basic", "premium", "enterprise", "pro", "admin"}
local user_counter = 0

request = function()
    user_counter = user_counter + 1
    local user_type = user_types[(user_counter % #user_types) + 1]
    local user_id = "user_" .. user_type .. "_" .. (user_counter % 1000)
    
    wrk.headers["X-User-ID"] = user_id
    wrk.headers["Content-Type"] = "application/json"
    wrk.headers["User-Agent"] = "ArbEdge-LoadTest/1.0"
    
    return wrk.format(nil, nil, nil, nil)
end

response = function(status, headers, body)
    if status ~= 200 then
        print("Error response: " .. status)
    end
end
