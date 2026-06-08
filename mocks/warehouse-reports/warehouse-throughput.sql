select
  w.id as warehouse_id,
  w.city as city,
  count(distinct o.id) as orders,
  count(distinct s.id) as shipments,
  sum(oi.quantity) as ordered_units
from warehouse_locations w
join warehouse_orders o on o.warehouse_id = w.id
join warehouse_order_items oi on oi.order_id = o.id
left join warehouse_shipments s on s.order_id = o.id
group by w.id, w.city
having sum(oi.quantity) > 0
order by ordered_units desc, city asc
