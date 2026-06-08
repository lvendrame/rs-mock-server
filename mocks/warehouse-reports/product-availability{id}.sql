select
  p.id as product_id,
  p.name as product,
  w.city as warehouse_city,
  i.on_hand as on_hand,
  i.reserved as reserved,
  i.reorder_point as reorder_point
from warehouse_products p
join warehouse_inventory i on i.product_id = p.id
join warehouse_locations w on w.id = i.warehouse_id
where p.id = ?
order by on_hand desc, warehouse_city asc
