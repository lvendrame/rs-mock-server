select
  o.id as order_id,
  c.name as customer,
  w.city as warehouse_city,
  p.name as product,
  oi.quantity as quantity,
  oi.unit_price as unit_price
from warehouse_orders o
join warehouse_customers c on c.id = o.customer_id
join warehouse_locations w on w.id = o.warehouse_id
join warehouse_order_items oi on oi.order_id = o.id
join warehouse_products p on p.id = oi.product_id
where o.id = ?
order by product asc
