select
  c.name as customer,
  w.city as warehouse_city,
  count(distinct o.id) as orders,
  sum(oi.quantity) as units
from warehouse_customers c
join warehouse_orders o on o.customer_id = c.id
join warehouse_locations w on w.id = o.warehouse_id
join warehouse_order_items oi on oi.order_id = o.id
join warehouse_products p on p.id = oi.product_id
where p.temperature_controlled = true
group by c.name, w.city
having sum(oi.quantity) >= 1
order by units desc, customer asc
