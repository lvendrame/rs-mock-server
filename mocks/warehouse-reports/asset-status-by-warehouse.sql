select
  w.id as warehouse_id,
  w.city as city,
  a.status as status,
  count(*) as assets
from warehouse_locations w
join warehouse_assets a on a.warehouse_id = w.id
group by w.id, w.city, a.status
having count(*) >= 1
order by city asc, assets desc, status asc
