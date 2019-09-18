DamageOccured = class("DamageOccured")

function DamageOccured:initialize(entityA, entityB)
    self.entities = {entityA, entityB}
end