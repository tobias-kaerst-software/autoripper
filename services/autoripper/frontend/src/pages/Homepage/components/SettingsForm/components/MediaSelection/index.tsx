import { Search } from 'lucide-react';
import { FC, ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '$/components/common/ui/button';
import { FormField } from '$/components/common/ui/form';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '$/components/common/ui/tabs';
import { MovieSelection } from '$/pages/Homepage/components/SettingsForm/components/MediaSelection/components/MovieSelection';
import { SeriesSelection } from '$/pages/Homepage/components/SettingsForm/components/MediaSelection/components/SeriesSelection';
import { MediaSelectionDrawer } from '$/pages/Homepage/components/SettingsForm/components/MediaSelectionDrawer';

import type { MetadataFormControl } from '$/pages/Homepage/components/SettingsForm';

interface Props {
  form: MetadataFormControl;
  children?: ReactNode;
}

export const MediaSelection: FC<Props> = ({ children, form }) => {
  const { t } = useTranslation();

  return (
    <FormField
      control={form.control}
      name='type'
      render={({ field }) => (
        <Tabs
          defaultValue={field.value}
          value={field.value}
          onValueChange={(v) => {
            field.onChange(v);
            form.resetField('selectedMedia');
            form.resetField('selectedSeason');
            form.resetField('selectedEpisodes');
          }}
        >
          <div className='flex gap-3'>
            <TabsList className='w-full'>
              <TabsTrigger value='movie' className='w-full'>
                {t('homepage.metadata.media.movie')}
              </TabsTrigger>
              <TabsTrigger value='tv_show' className='w-full'>
                {t('homepage.metadata.media.tvShow')}
              </TabsTrigger>
            </TabsList>

            <FormField
              control={form.control}
              name='selectedMedia'
              render={({ field: mediaField }) => (
                <div className='flex gap-2'>
                  <MediaSelectionDrawer
                    type={field.value}
                    onMediaSelect={(v) => {
                      form.resetField('selectedSeason');
                      form.resetField('selectedEpisodes');
                      mediaField.onChange(v);
                    }}
                  >
                    <Button
                      className='aspect-square h-full p-3'
                      type='button'
                      variant={form.getFieldState('selectedMedia').error ? 'destructive' : 'default'}
                    >
                      <Search className='size-4' />
                    </Button>
                  </MediaSelectionDrawer>
                  {children}
                </div>
              )}
            />
          </div>

          <TabsContent value='movie' tabIndex={-1}>
            <MovieSelection form={form} />
          </TabsContent>
          <TabsContent value='tv_show' tabIndex={-1}>
            <SeriesSelection form={form} />
          </TabsContent>
        </Tabs>
      )}
    />
  );
};