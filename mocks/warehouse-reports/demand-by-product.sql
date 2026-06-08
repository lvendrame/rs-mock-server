select
  p.category as category,
  p.name as product,
  count(distinct oi.order_id) as orders,
  sum(oi.quantity) as units
from warehouse_products p
join warehouse_order_items oi on oi.product_id = p.id
group by p.category, p.name
having sum(oi.quantity) > 20
order by units desc, product asc
limit 10
