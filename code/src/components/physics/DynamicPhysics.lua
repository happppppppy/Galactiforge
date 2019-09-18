local DynamicPhysics = Component.create("DynamicPhysics")

function DynamicPhysics:initialize(linear_force_strength, angular_torque_strength, linear_speed_max, angular_speed_max, linear_damping, angular_damping, start_vel, x_vel, y_vel, mass)
  self.mass = mass or 0

  self.linear_force_strength = linear_force_strength
  self.angular_torque_strength = angular_torque_strength
  
  self.linear_speed_max = linear_speed_max
  self.angular_speed_max = angular_speed_max

  self.linear_damping = linear_damping
  self.angular_damping = angular_damping

  self.start_vel = start_vel or 0
  self.x_vel = x_vel or 0
  self.y_vel = y_vel or 0
end