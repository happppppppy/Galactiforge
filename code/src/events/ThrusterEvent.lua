ThrusterEvent = class("ThrusterEvent")

function ThrusterEvent:initialize(thruster, dt)
    self.thruster = thruster
    self.dt = dt
end