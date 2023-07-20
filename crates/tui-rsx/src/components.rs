use crate::prelude::*;

#[component]
pub fn Popup<T, B, V>(
    _cx: T,
    #[prop(children)] children: V,
    percent_x: u16,
    percent_y: u16,
) -> impl View<B>
where
    T: Clone + 'static,
    B: Backend + 'static,
    V: LazyView<B> + Clone + 'static,
{
    move || {
        let mut children = children.clone();
        view! {
            <column>
                <row percentage=(100 - percent_y)/2 />
                <row percentage=percent_y>
                    <column percentage=(100 - percent_x)/2 />
                    <column percentage=percent_x>
                        <overlay>
                            <clear/>
                            {children}
                        </overlay>
                    </column>
                    <column percentage=(100 - percent_x)/2 />
                </row>
                <row percentage=(100 - percent_y)/2 />
            </column>
        }
    }
}
