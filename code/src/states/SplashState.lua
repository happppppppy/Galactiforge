local splashy = require("code/lib/splashy")

local SplashState = {}

function SplashState:init()
  splashy.addSplash(love.graphics.newImage("assets/images/SplashScreen.png"))
  splashy.onComplete(function() Gamestate.switch(MenuState) end)

end

function SplashState:update(dt)
  splashy.update(dt)
end

function SplashState:draw()
  splashy.draw()
end

return SplashState