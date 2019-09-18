local CollisionPhysics = Component.create("CollisionPhysics")

function CollisionPhysics:initialize(shape, x, y, restitution, category, mask)
  self.shape = shape
  self.restitution = restitution
  self.x = x
  self.y = y
  self.category = category
  self.mask = mask
end