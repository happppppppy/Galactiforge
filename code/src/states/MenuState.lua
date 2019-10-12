local MenuState = {}

function MenuState:init()
  loveframes = require("code/lib/loveframes")
  height = love.graphics.getHeight()
  local newgame_button = loveframes.Create("button")
  newgame_button:SetWidth(200)
  newgame_button:SetText("New Game")
  newgame_button:SetY(height - 120)
  newgame_button:SetX(20, false)
  newgame_button.OnClick = function()
    Gamestate.switch(MainState)
  end

  local load_button = loveframes.Create("button")
  load_button:SetWidth(200)
  load_button:SetText("Load Game")
  load_button:SetY(height - 90)
  load_button:SetX(20, false)
  load_button.OnClick = function()
    Gamestate.switch(MainState)
  end

  local exit_button = loveframes.Create("button")
  exit_button:SetWidth(200)
  exit_button:SetText("Exit")
  exit_button:SetY(height - 60)
  exit_button:SetX(20, false)
  exit_button.OnClick = function()
    love.event.quit( exitstatus )
  end

  menu_font = love.graphics.newFont("assets/fonts/GiantRobotArmy-Medium.ttf", 80)

end

function MenuState:update(dt)
  loveframes.update(dt)
end

function MenuState:draw()
  
  love.graphics.setFont( menu_font )

  love.graphics.print("Galactiforge", 90, 20, math.rad(90), 1, 1)
  loveframes.draw()
end

function MenuState:mousepressed(x, y, button)
  loveframes.mousepressed(x, y, button)
end

function MenuState:mousereleased(x, y, button)
  loveframes.mousereleased(x, y, button)
end

function MenuState:keypressed(key, unicode)
  if key == "escape" then Gamestate.switch(MainState) end
  loveframes.keypressed(key, unicode)
end

function MenuState:keyreleased(key)
  loveframes.keyreleased(key)
end

return MenuState