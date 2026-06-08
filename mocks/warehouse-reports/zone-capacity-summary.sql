select
  w.city as city,
  z.kind as zone_kind,
  count(*) as zones,
  sum(z.capacity) as capacity,
  avg(z.capacity) as avg_capacity
from warehouse_locations w
join warehouse_zones z on z.warehouse_id = w.id
group by w.city, z.kind
having sum(z.capacity) > 2000
order by city asc, capacity desc
