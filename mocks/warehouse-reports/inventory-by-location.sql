select
  w.id as warehouse_id,
  w.city as city,
  count(distinct i.product_id) as skus,
  sum(i.on_hand) as on_hand,
  sum(i.reserved) as reserved,
  avg(i.reorder_point) as avg_reorder_point
from warehouse_locations w
join warehouse_inventory i on i.warehouse_id = w.id
group by w.id, w.city
having sum(i.on_hand) > 0
order by reserved desc, on_hand desc, city asc
