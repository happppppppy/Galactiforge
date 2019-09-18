local Sprite = Component.create("Sprite")

function Sprite:initialize(imagePath, r, sx, sy, ox, oy, centred)
  self.image = love.graphics.newImage(imagePath)
  self.r = r
  if sx then self.sx = sx  end
  if sy then self.sy = sy  end
  if ox then self.ox = ox  end
  if oy then self.oy = oy  end
  self.width = self.image:getWidth()
  self.height = self.image:getHeight()
  if centred == true then
    self.ox = self.width/2
    self.oy = self.height/2
  end
end