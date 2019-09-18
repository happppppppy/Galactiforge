CollisionStart = class("CollisionStart")

function CollisionStart:initialize(a, b, coll)
    self.a = a
    self.b = b
    self.coll = coll
end