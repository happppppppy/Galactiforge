FireEvent = class("FireEvent")

function FireEvent:initialize(weapon, dt)
    self.weapon = weapon
    self.dt = dt
end