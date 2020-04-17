-- Importing 3rd party libraries
local lovetoys = require "code/lib/lovetoys"
lovetoys.initialize({
	globals = true, 
	debug = true
})
loveframes = require("code/lib/loveframes")

Gamestate = require("code/lib/hump/gamestate")

SplashState = require("code/src/states/SplashState")
MainState = require("code/src/states/MainState")
MenuState = require("code/src/states/MenuState")

function love.load()
	Gamestate.registerEvents()
	-- Gamestate.switch(SplashState)
	
	Gamestate.switch(MainState)
end

function love.update(dt)
	loveframes.update(dt)
end

function love.draw()
	loveframes.draw()
end

function love.mousepressed(x, y, button)
 
	-- your code

	loveframes.mousepressed(x, y, button)

end

function love.mousereleased(x, y, button)

	-- your code

	loveframes.mousereleased(x, y, button)

end

function love.keypressed(key, unicode)

	-- your code

	loveframes.keypressed(key, unicode)

end

function love.keyreleased(key)

	-- your code

	loveframes.keyreleased(key)

end