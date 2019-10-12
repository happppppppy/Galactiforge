-- Importing 3rd party libraries
local lovetoys = require "code/lib/lovetoys"
lovetoys.initialize({
	globals = true, 
	debug = true
})

Gamestate = require("code/lib/hump/gamestate")

SplashState = require("code/src/states/SplashState")
MainState = require("code/src/states/MainState")

function love.load()
	Gamestate.registerEvents()
	Gamestate.switch(SplashState)
end

function love.update(dt)
end

function love.draw()
end