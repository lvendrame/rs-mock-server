select
  c.id as customer_id,
  c.name as customer,
  c.tier as tier,
  count(distinct o.id) as orders,
  count(*) as lines,
  sum(oi.quantity) as units
from warehouse_customers c
join warehouse_orders o on o.customer_id = c.id
join warehouse_order_items oi on oi.order_id = o.id
group by c.id, c.name, c.tier
having sum(oi.quantity) > 20
order by units desc, orders desc, customer asc
