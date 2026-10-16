# Authoring helper: regenerates assets/content/core.ron and assets/content/blueprints/ships.ron.
# Run from the repository root: python3 tools/gen_content.py
# Authoring aid: emits assets/content/*.ron (not committed).
R = []
def res(id, kind="Item", **kw):
    extra = "".join(f", {k}: {v}" for k, v in kw.items())
    R.append(f'    (id: "{id}", name: "{id.replace("_"," ").title()}", kind: {kind}{extra}),')
res("ore"); res("ice", ammo="Some((damage_mult: 0.2, frost: 0.6))"); res("rock_dust")
res("titanstone"); res("oilstone"); res("plutonium", flammability=0.3)
res("titanium_ore"); res("molten_titanium","Fluid")
res("casings"); res("plates"); res("powder", flammability=0.8); res("circuits")
res("shells", flammability=0.4, ammo="Some((damage_mult: 1.0))")
res("ap_shells", flammability=0.3, ammo="Some((damage_mult: 1.2, pierce: 3))")
res("incendiary_shells", flammability=0.9, ammo="Some((damage_mult: 0.8, incendiary: 0.8))")
res("explosive_shells", flammability=1.0, ammo="Some((damage_mult: 1.1, splash_radius: 2.0))")
res("rockets", flammability=1.0, ammo="Some((damage_mult: 1.0, splash_radius: 2.5))")
res("data_core", research_value=10.0)
res("fuel_oil","Fluid", flammability=0.6); res("hydrogen","Fluid", flammability=0.9)
res("oxygen","Fluid", flammability=0.5); res("water","Fluid")
res("fuel","Fluid", flammability=1.0); res("glycerol","Fluid", flammability=0.2)
res("coolant","Fluid", ammo="Some((damage_mult: 0.1, frost: 1.0))")
res("sample_guidance", research_value=0.0); res("sample_reactor"); res("sample_alien_alloy")

REC = []
def rec(id, ins, outs, t, p=0):
    f = lambda l: "[" + ", ".join(f'("{a}", {float(b)})' for a, b in l) + "]"
    REC.append(f'    (id: "{id}", inputs: {f(ins)}, outputs: {f(outs)}, time: {t}, power: {float(p)}),')
rec("crush_titanstone", [("ore",1)], [("titanstone",1)], 0.5, 5)
rec("crush_oilstone", [("ore",1)], [("oilstone",1)], 0.5, 5)
rec("crush_plutonium", [("ore",2)], [("plutonium",1)], 1.0, 8)
rec("separate_titanium", [("titanstone",1)], [("titanium_ore",1)], 0.5, 5)
rec("separate_oilstone", [("oilstone",1)], [("powder",1),("fuel_oil",2)], 0.5, 5)
rec("melt_titanium", [("titanium_ore",1)], [("molten_titanium",1)], 0.5, 20)
rec("electrolysis", [("ice",1)], [("hydrogen",2),("oxygen",1)], 0.5, 20)
rec("melt_ice", [("ice",1)], [("water",4)], 0.5, 10)
rec("cast_casings", [("molten_titanium",1)], [("casings",2)], 0.5, 10)
rec("cast_plates", [("molten_titanium",2)], [("plates",1)], 0.75, 10)
rec("refine_fuel", [("fuel_oil",1),("hydrogen",1)], [("fuel",4)], 0.5, 10)
rec("refine_glycerol", [("fuel_oil",1),("hydrogen",1)], [("glycerol",2)], 0.5, 10)
rec("make_coolant", [("glycerol",1),("water",1)], [("coolant",4)], 0.5, 10)
rec("assemble_shells", [("casings",1),("powder",1)], [("shells",2)], 0.5, 10)
rec("assemble_ap_shells", [("casings",1),("powder",1),("plates",1)], [("ap_shells",2)], 0.75, 12)
rec("assemble_incendiary", [("casings",1),("powder",1),("fuel_oil",2)], [("incendiary_shells",2)], 0.75, 12)
rec("assemble_explosive", [("casings",1),("powder",1),("plutonium",1)], [("explosive_shells",2)], 0.75, 12)
rec("make_circuits", [("plates",1),("plutonium",1)], [("circuits",2)], 1.0, 15)
rec("assemble_rockets", [("casings",2),("powder",2),("circuits",1)], [("rockets",1)], 1.0, 15)
rec("make_data_core", [("circuits",1),("plates",1)], [("data_core",1)], 2.0, 20)

B = []
def blk(id, cat, sprite, kind, mass=1.0, hp=20, armor=0, cost=None, unlocked=True, buildable=True, flam=0.0, sample=None):
    cost = cost if cost is not None else [("plates",1)]
    c = "[" + ", ".join(f'("{a}", {b})' for a,b in cost) + "]"
    s = f', sample: Some("{sample}")' if sample else ""
    B.append(f'    (id: "{id}", name: "{id.replace("_"," ").title()}", category: {cat}, sprite: {sprite}, mass: {mass}, hp: {float(hp)}, armor: {float(armor)}, cost: {c}, buildable: {str(buildable).lower()}, starts_unlocked: {str(unlocked).lower()}, flammability: {flam}{s},\n      kind: {kind}),')
def sp(tile, frames=0, fps=0, base=None, tint=None):
    s = f"(tile: {tile}"
    if frames: s += f", frames: {frames}, fps: {float(fps)}"
    if base is not None: s += f", base: Some({base})"
    if tint: s += f", tint: Some(({tint[0]}, {tint[1]}, {tint[2]}))"
    return s + ")"
blk("ship_core","Core",sp(120),"Core",mass=5,hp=250,armor=2,cost=[],buildable=False)
blk("armor_full","Hull",sp(113),"Hull",mass=2,hp=120,armor=4)
blk("armor_half","Hull",sp(110),"Hull",mass=1.5,hp=80,armor=3)
blk("armor_corner","Hull",sp(111),"Hull",mass=1.5,hp=80,armor=3)
blk("armor_corner_half","Hull",sp(112),"Hull",mass=1,hp=50,armor=2)
blk("floor","Hull",sp(114),"Hull",mass=0.5,hp=20,cost=[])
blk("conveyor","Logistics",sp(130,6,12),"Conveyor(speed: 8.0)",mass=0.5,hp=15)
blk("splitter","Logistics",sp(130,6,12,tint=(1.0,0.8,0.5)),"Splitter(speed: 8.0)",mass=0.5,hp=15)
blk("container","Logistics",sp(144),"Storage(capacity: 400)",mass=2,hp=40,cost=[("plates",2)])
blk("ore_container","Logistics",sp(142),'Storage(capacity: 300, filter: ["ore"])',mass=2,hp=40,cost=[("plates",2)])
blk("ice_container","Logistics",sp(141),'Storage(capacity: 300, filter: ["ice"])',mass=2,hp=40,cost=[("plates",2)])
blk("shell_container","Logistics",sp(140),'Storage(capacity: 200, filter: ["shells", "ap_shells", "incendiary_shells", "explosive_shells", "rockets"])',mass=2,hp=40,cost=[("plates",2)],flam=0.2)
blk("fuel_container","Logistics",sp(143),'Tank(capacity: 400.0, filter: ["fuel", "oxygen"])',mass=2,hp=40,cost=[("plates",2)],flam=0.3)
blk("fluid_tank","Logistics",sp(143,tint=(0.6,0.8,1.0)),'Tank(capacity: 400.0)',mass=2,hp=40,cost=[("plates",2)])
blk("pipe","Logistics",sp(152),"Pipe",mass=0.3,hp=10)
blk("cable","Power",sp(152,tint=(1.0,0.85,0.3)),"Cable",mass=0.2,hp=8)
blk("crusher","Factory",sp(30,7,10),'Machine(recipes: ["crush_titanstone", "crush_oilstone", "crush_plutonium"])',mass=4,hp=40,cost=[("plates",4)])
blk("separator","Factory",sp(40,7,10),'Machine(recipes: ["separate_titanium", "separate_oilstone"])',mass=4,hp=40,cost=[("plates",4)])
blk("arc_furnace","Factory",sp(50,7,10),'Machine(recipes: ["melt_titanium", "electrolysis", "melt_ice"])',mass=5,hp=40,cost=[("plates",6)])
blk("metal_injection_molding","Factory",sp(60,7,10),'Machine(recipes: ["cast_casings", "cast_plates"])',mass=4,hp=40,cost=[("plates",4)])
blk("fractionator","Factory",sp(20,7,10),'Machine(recipes: ["refine_fuel", "refine_glycerol"])',mass=4,hp=40,cost=[("plates",4)],flam=0.3)
blk("centrifuge","Factory",sp(70,7,10),'Machine(recipes: ["make_coolant"])',mass=4,hp=40,cost=[("plates",4)])
blk("assembly_centre","Factory",sp(70,7,10,tint=(1.0,0.9,0.7)),'Machine(recipes: ["assemble_shells", "assemble_ap_shells", "assemble_incendiary", "assemble_explosive", "assemble_rockets"])',mass=4,hp=40,cost=[("plates",4)])
blk("fabricator","Factory",sp(10,7,10),'Machine(recipes: ["make_circuits", "make_data_core"])',mass=4,hp=40,cost=[("plates",6),("casings",2)],unlocked=False)
blk("fuel_generator","Power",sp(50,7,10,tint=(1.0,0.75,0.4)),'Generator(output: 80.0, fuel: Some(("fuel", 0.5)))',mass=4,hp=40,cost=[("plates",5)],flam=0.5)
blk("reactor","Power",sp(120,tint=(0.6,1.0,0.6)),'Generator(output: 400.0, fuel: Some(("plutonium", 0.05)))',mass=10,hp=120,armor=3,cost=[("plates",20),("circuits",10)],unlocked=False,sample="sample_reactor")
blk("solar_panel","Power",sp(114,tint=(0.5,0.6,1.0)),"Solar(output: 6.0)",mass=0.5,hp=10,unlocked=False)
blk("battery","Power",sp(144,tint=(1.0,0.9,0.4)),"Battery(capacity: 600.0, rate: 100.0)",mass=3,hp=30,cost=[("plates",3)])
cannon_growth = "[(size: (1, 1), demand: 0.5), (size: (2, 2), demand: 1.5), (size: (3, 3), demand: 3.5), (size: (4, 4), demand: 7.0), (size: (5, 5), demand: 12.0)]"
blk("cannon","Weapon",sp(80,7,30,base=113),
    f'Weapon((kind: Cannon, ammo: ["shells", "ap_shells", "incendiary_shells", "explosive_shells", "ice", "coolant"], growth: {cannon_growth}, damage: 10.0, interval: 0.6, projectile_speed: 30.0, range: 40.0, heat_per_shot: 6.0, heat_capacity: 100.0, cool_rate: 12.0, turn_rate: 3.0))',
    mass=3,hp=60,armor=2,cost=[("plates",4),("casings",2)])
rocket_growth = "[(size: (1, 1), demand: 0.2), (size: (2, 2), demand: 0.5), (size: (3, 3), demand: 1.0), (size: (4, 4), demand: 1.8), (size: (5, 5), demand: 3.0)]"
blk("rocket_launcher","Weapon",sp(80,7,30,base=113,tint=(1.0,0.6,0.55)),
    f'Weapon((kind: Rocket, ammo: ["rockets"], growth: {rocket_growth}, damage: 45.0, interval: 2.0, projectile_speed: 16.0, range: 60.0, heat_per_shot: 15.0, power: 5.0, turn_rate: 2.0, homing: 2.5))',
    mass=4,hp=70,armor=2,cost=[("plates",6),("circuits",4)],unlocked=False,sample="sample_guidance")
laser_growth = "[(size: (1, 1), demand: 15.0), (size: (2, 2), demand: 40.0), (size: (3, 3), demand: 90.0)]"
blk("mining_laser","Utility",sp(80,tint=(0.6,1.0,0.9),base=113),
    f'Weapon((kind: MiningLaser, growth: {laser_growth}, damage: 20.0, interval: 0.1, projectile_speed: 0.0, range: 25.0, heat_per_shot: 0.5, power: 15.0, turn_rate: 2.0))',
    mass=3,hp=40,cost=[("plates",5),("circuits",3)],unlocked=False)
lt = "[(size: (1, 1), demand: 0.6), (size: (1, 2), demand: 1.5), (size: (2, 2), demand: 3.0), (size: (2, 3), demand: 5.0), (size: (3, 3), demand: 8.0)]"
blk("large_thruster","Thruster",sp(0),f'Thruster((fuel: "fuel", oxidizer: Some("oxygen"), growth: {lt}, thrust: 120.0))',mass=3,hp=50,armor=1,cost=[("plates",4)],flam=0.3)
mt = "[(size: (1, 1), demand: 0.3), (size: (1, 2), demand: 0.8)]"
blk("mini_thruster","Thruster",sp(1),f'Thruster((fuel: "fuel", growth: {mt}, thrust: 50.0))',mass=1.5,hp=30,cost=[("plates",2)],flam=0.3)
blk("drill","Utility",sp(30,tint=(0.75,0.75,0.75)),"Drill(rate: 25.0, power: 10.0)",mass=3,hp=60,armor=2,cost=[("plates",4)])
blk("salvager","Utility",sp(40,7,8,tint=(0.6,1.0,0.6)),"Salvager(rate: 20.0, power: 10.0)",mass=3,hp=50,cost=[("plates",4)])
blk("tractor_beam","Utility",sp(150,4,6),"Tractor(range: 30.0, force: 250.0, power: 25.0)",mass=3,hp=30,cost=[("plates",5)],unlocked=False)
blk("radar","Utility",sp(150,4,6,tint=(0.6,1.0,0.6)),"Radar(range: 200.0)",mass=2,hp=20,cost=[("plates",3)],unlocked=False)
blk("research_lab","Research",sp(10,7,6,tint=(0.8,0.8,1.0)),"Lab(power: 20.0, rate: 1.0)",mass=4,hp=40,cost=[("plates",6)])
def mineral(id, tint, yields, hp):
    y = "[" + ", ".join(f'("{a}", {float(b)})' for a,b in yields) + "]"
    blk(id,"Mineral",sp(113,tint=tint),f"Mineral(yields: {y})",mass=4,hp=hp,cost=[],buildable=False,unlocked=False)
mineral("rock",(0.55,0.45,0.4),[("ore",0.5)],40)
mineral("ore_vein",(0.9,0.55,0.3),[("ore",3)],50)
mineral("ice_vein",(0.6,0.85,1.0),[("ice",3)],30)
mineral("rich_vein",(0.8,0.5,1.0),[("ore",5),("plutonium",1)],80)

RS = '''    (id: "advanced_fabrication", name: "Advanced Fabrication", cost: 60.0, unlocks: ["fabricator", "solar_panel"]),
    (id: "sensors", name: "Sensors", cost: 40.0, unlocks: ["radar"]),
    (id: "salvage_ops", name: "Salvage Operations", cost: 50.0, unlocks: ["tractor_beam"]),
    (id: "mining_lasers", name: "Mining Lasers", cost: 120.0, requires: ["advanced_fabrication"], unlocks: ["mining_laser"]),
    (id: "rocketry", name: "Rocketry", cost: 200.0, requires: ["advanced_fabrication"], samples: ["sample_guidance"], unlocks: ["rocket_launcher"]),
    (id: "fission_power", name: "Fission Power", cost: 250.0, requires: ["advanced_fabrication"], samples: ["sample_reactor"], unlocks: ["reactor"]),'''
MS = '''    (id: "first_blood", name: "First Blood", objective: DestroyShips(count: 1), reward: (research_points: 50.0)),
    (id: "scrap_collector", name: "Scrap Collector", objective: SalvageBlocks(count: 25), reward: (research_points: 40.0, samples: ["sample_alien_alloy"])),
    (id: "prospector", name: "Prospector", objective: MineResource(resource: "ore", amount: 150.0), reward: (research_points: 40.0)),
    (id: "holdout", name: "Holdout", objective: Survive(seconds: 300.0), reward: (unlocks: ["radar"]))'''
out = "// Galactiforge core content. Tile indices refer to assets/images/galactiforge_tilesets/factory_devices.png\n// (32x32 tiles, 10 columns, 0-based row-major).\n(\n  resources: [\n" + "\n".join(R) + "\n  ],\n  recipes: [\n" + "\n".join(REC) + "\n  ],\n  blocks: [\n" + "\n".join(B) + "\n  ],\n  research: [\n" + RS + "\n  ],\n  missions: [\n" + MS + "\n  ],\n)\n"
open("assets/content/core.ron","w").write(out)

# ---------------- blueprints ----------------
def bp(id, blocks, stock):
    lines = []
    for b in blocks:
        name, x, y = b[0], b[1], b[2]
        rot = b[3] if len(b) > 3 else 0
        rec = f', recipe: Some("{b[4]}")' if len(b) > 4 and b[4] else ""
        lines.append(f'      (block: "{name}", x: {x}, y: {y}, rot: {rot}{rec}),')
    st = "[" + ", ".join(f'("{a}", {float(q)})' for a,q in stock) + "]"
    return f'    (id: "{id}", name: "{id.replace("_"," ").title()}",\n     blocks: [\n' + "\n".join(lines) + f"\n     ],\n     stock: {st}),"
forge = [
 ("armor_corner",-1,5,1),("drill",0,5,0),("ore_container",1,5),
 ("cannon",-2,4,2),("shell_container",-1,4),("cable",0,4),("crusher",1,4,0,"crush_titanstone"),("cannon",2,4,1),
 ("mini_thruster",-3,3,3),("armor_full",-2,3),("armor_full",-1,3),("cable",0,3),("container",1,3),("shell_container",2,3),("mini_thruster",3,3,1),
 ("fuel_container",-3,2),("armor_full",-2,2),("fuel_container",-1,2),("battery",0,2),("container",1,2),("armor_full",2,2),("fuel_container",3,2),
 ("mini_thruster",-2,1,3),("fuel_container",-1,1),("ship_core",0,1),("fuel_container",1,1),("mini_thruster",2,1,1),
 ("armor_full",-2,0),("pipe",-1,0),("pipe",0,0),("pipe",1,0),("armor_full",2,0),
 ("large_thruster",-1,-1,0),("large_thruster",1,-1,0),
]
bandit = [
 ("ship_core",0,0),("large_thruster",0,3,2),("fuel_container",0,2),("fuel_container",0,-1),
 ("mini_thruster",-1,-1,3),("mini_thruster",1,-1,1),("large_thruster",0,-2,0),
 ("cannon",-1,1,2),("cannon",1,1,1),("shell_container",0,1),
 ("armor_corner",1,3,1),("armor_corner",-1,3,0),("armor_corner",1,-2,2),("armor_corner",-1,-2,3),
 ("armor_full",1,2),("armor_full",-1,2),("armor_full",1,0),("armor_full",-1,0),
]
raider = [
 ("ship_core",0,0),("cable",-1,0),("cable",1,0),
 ("rocket_launcher",-2,1,2),("shell_container",-1,1),("shell_container",0,1),("shell_container",1,1),("rocket_launcher",2,1,1),
 ("armor_corner",-1,2,1),("cannon",0,2,1),("armor_corner",1,2,0),
 ("cable",-2,0),("cable",2,0),
 ("fuel_container",-1,-1),("pipe",0,-1),("fuel_container",1,-1),
 ("large_thruster",-1,-2,0),("large_thruster",1,-2,0),("armor_full",0,-2),
 ("mini_thruster",-2,-1,3),("mini_thruster",2,-1,1),
]
hauler = [
 ("ship_core",0,0),("reactor",0,1),("container",-1,0),("container",1,0),("cable",0,-1),
 ("armor_full",-1,1),("armor_full",1,1),("armor_full",-1,-1),("armor_full",1,-1),
 ("fuel_container",0,-2),("large_thruster",0,-3,0),
]
out = "// Ship blueprints. Local frame: +y = ship forward, rot = CCW quarter turns.\n(\n  blueprints: [\n" + "\n".join([
  bp("forge_runner", forge, [("fuel",900),("oxygen",100),("shells",350),("plates",200),("casings",40),("ore",50)]),
  bp("micro_bandit", bandit, [("fuel",500),("shells",150)]),
  bp("raider", raider, [("fuel",600),("rockets",60),("shells",200)]),
  bp("wreck_hauler", hauler, [("plutonium",20),("plates",40),("circuits",10)]),
]) + "\n  ],\n)\n"
open("assets/content/blueprints/ships.ron","w").write(out)
