select
  country,
  count(*) as warehouses,
  sum(capacity) as total_capacity,
  avg(capacity) as avg_capacity,
  min(capacity) as min_capacity,
  max(capacity) as max_capacity
from warehouse_locations
group by country
having count(*) >= 1
order by total_capacity desc, country asc
