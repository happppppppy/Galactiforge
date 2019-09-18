local DamageSystem = class("DamageSystem", System)

function DamageSystem:fireEvent(event)
  for i,v in pairs(event.entities) do
    if v:get("Health") ~= nil then
      health = v:get("Health")
    end

    if v:get("Bullet") ~= nil then
      bullet = v:get("Bullet")
    end
  end

  if health ~= nil and bullet ~= nil then
    bullet.impact = true
    if bullet.damage ~= nil then
      health.health = health.health - bullet.damage
    end
  end

  if health == nil and bullet ~= nil then
    bullet.impact = true
  end

end

return DamageSystem